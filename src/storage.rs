use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::database::{
    self as sqlx, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use anyhow::{Context, Result, bail};
use directories::ProjectDirs;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::recurrence::RecoveryPlan;

static MIGRATOR: LazyLock<sqlx_core::migrate::Migrator> = LazyLock::new(|| {
    use sqlx_core::migrate::{Migration, MigrationType, Migrator};
    let migrations = [
        (
            1,
            "initial schema",
            include_str!("../migrations/0001_initial_schema.sql"),
        ),
        (
            2,
            "schedule evaluation cursor",
            include_str!("../migrations/0002_schedule_evaluation_cursor.sql"),
        ),
        (
            3,
            "operational queries",
            include_str!("../migrations/0003_operational_queries.sql"),
        ),
    ]
    .into_iter()
    .map(|(version, description, sql)| {
        Migration::new(
            version,
            description.into(),
            MigrationType::Simple,
            sql.into(),
            false,
        )
    })
    .collect::<Vec<_>>();
    Migrator {
        migrations: migrations.into(),
        ..Migrator::DEFAULT
    }
});

const DEFAULT_TRIGGER: &str = r#"{"enabled":false,"scheduleBehavior":"coexist"}"#;
const DEFAULT_RETRY_POLICY: &str = r#"{"maxRetries":3,"delaysSeconds":[60,300,900]}"#;
const MAX_PARAMETERS_BYTES: usize = 64 * 1024;
const HISTORY_RETENTION_MICROSECONDS: i64 = 90 * 24 * 60 * 60 * 1_000_000;

#[derive(Debug, Clone, Serialize)]
pub struct NewSchedule {
    pub name: Option<String>,
    pub program: String,
    pub args: Vec<String>,
    pub timing: Value,
    pub parameters: Option<Value>,
    pub missed_policy: String,
    pub parameter_update_trigger: Option<Value>,
    pub retry_policy: Option<Value>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateScheduleOutcome {
    Created { id: String },
    Replayed { id: String },
}

#[derive(Debug, Clone)]
pub struct ActiveSchedule {
    pub id: String,
    pub timing: Value,
    pub missed_policy: String,
    pub last_evaluated_at: i64,
}

pub async fn active_schedules(pool: &SqlitePool) -> Result<Vec<ActiveSchedule>> {
    let rows = sqlx::query_as::<_, (String, String, String, i64)>(
        "SELECT id, timing_json, missed_policy, last_evaluated_at \
         FROM schedules WHERE status = 'active' ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .context("could not load active schedules")?;
    rows.into_iter()
        .map(|(id, timing, missed_policy, last_evaluated_at)| {
            Ok(ActiveSchedule {
                id,
                timing: serde_json::from_str(&timing).context("stored timing is invalid")?,
                missed_policy,
                last_evaluated_at,
            })
        })
        .collect()
}

pub async fn persist_scheduled_occurrence(
    pool: &SqlitePool,
    schedule_id: &str,
    expected_cursor: i64,
    scheduled_for: i64,
) -> Result<bool> {
    let mut tx = pool
        .begin()
        .await
        .context("could not start scheduled occurrence transaction")?;
    let result = sqlx::query(
        "UPDATE schedules SET last_evaluated_at = ?, updated_at = ? \
         WHERE id = ? AND last_evaluated_at = ? AND status = 'active'",
    )
    .bind(scheduled_for)
    .bind(unix_now()?)
    .bind(schedule_id)
    .bind(expected_cursor)
    .execute(&mut *tx)
    .await
    .context("could not advance scheduled occurrence cursor")?;
    if result.rows_affected() == 0 {
        tx.rollback()
            .await
            .context("could not roll back stale occurrence transaction")?;
        return Ok(false);
    }
    sqlx::query(
        "INSERT INTO occurrences (id, schedule_id, status, source, scheduled_for, created_at) \
         VALUES (?, ?, 'queued', 'schedule', ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(schedule_id)
    .bind(scheduled_for)
    .bind(unix_now()?)
    .execute(&mut *tx)
    .await
    .context("could not persist scheduled occurrence")?;
    tx.commit()
        .await
        .context("could not commit scheduled occurrence")?;
    Ok(true)
}

pub async fn due_occurrences(pool: &SqlitePool, now: i64) -> Result<Vec<String>> {
    sqlx::query_scalar("SELECT o.id FROM occurrences o JOIN schedules s ON s.id=o.schedule_id WHERE o.status = 'queued' AND o.scheduled_for <= ? AND (o.source!='retry' OR s.status!='cancelled') AND (s.missed_policy!='sequence' OR NOT EXISTS (SELECT 1 FROM occurrences prior WHERE prior.schedule_id=o.schedule_id AND prior.status IN ('queued','running') AND (prior.scheduled_for<o.scheduled_for OR (prior.scheduled_for=o.scheduled_for AND prior.id<o.id)))) AND (s.missed_policy!='sequence' OR NOT EXISTS (SELECT 1 FROM occurrences running WHERE running.schedule_id=o.schedule_id AND running.status='running')) ORDER BY o.scheduled_for, o.created_at")
        .bind(now)
        .fetch_all(pool)
        .await
        .context("could not load due occurrences")
}

pub async fn mark_running_occurrences_interrupted(pool: &SqlitePool, now: i64) -> Result<u64> {
    let result = sqlx::query(
        "UPDATE occurrences SET status = 'interrupted', finished_at = ? WHERE status = 'running'",
    )
    .bind(now)
    .execute(pool)
    .await
    .context("could not mark interrupted occurrences after restart")?;
    Ok(result.rows_affected())
}

pub fn default_database_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("COMMAND_SCHEDULER_DB_PATH") {
        return Ok(PathBuf::from(path));
    }

    let directories = ProjectDirs::from("com", "Command Scheduler", "Command Scheduler")
        .context("could not locate the current user's local data directory")?;
    Ok(directories.data_local_dir().join("scheduler.sqlite3"))
}

pub fn acquire_daemon_lock() -> Result<std::fs::File> {
    use fs2::FileExt;
    let path = default_database_path()?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let canonical = if path.exists() {
        path.canonicalize()?
    } else {
        parent
            .canonicalize()?
            .join(path.file_name().context("database path must name a file")?)
    };
    let mut lock_name = canonical.as_os_str().to_os_string();
    lock_name.push(".lock");
    let lock_path = PathBuf::from(lock_name);
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .with_context(|| format!("could not open daemon lock {}", lock_path.display()))?;
    lock.try_lock_exclusive()
        .context("another Command Scheduler daemon already owns this database")?;
    Ok(lock)
}

pub async fn connect_default() -> Result<SqlitePool> {
    connect_at(&default_database_path()?).await
}

pub async fn connect_at(path: &Path) -> Result<SqlitePool> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("could not create database directory {}", parent.display()))?;
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .with_context(|| format!("could not open SQLite database {}", path.display()))?;

    MIGRATOR
        .run(&pool)
        .await
        .context("could not apply SQLite migrations")?;
    Ok(pool)
}

pub async fn create_schedule(
    pool: &SqlitePool,
    schedule: &NewSchedule,
) -> Result<CreateScheduleOutcome> {
    validate_schedule(schedule)?;
    let fingerprint =
        hex_digest(&serde_json::to_vec(schedule).context("could not encode schedule request")?);
    let parameters_json =
        serialize_parameters(schedule.parameters.as_ref().unwrap_or(&Value::Null))?;
    let args_json =
        serde_json::to_string(&schedule.args).context("could not encode command arguments")?;
    let timing_json = serialize_json(&schedule.timing)?;
    let trigger_json = match &schedule.parameter_update_trigger {
        Some(value) => serialize_json(value)?,
        None => DEFAULT_TRIGGER.to_owned(),
    };
    let retry_json = match &schedule.retry_policy {
        Some(value) => serialize_json(value)?,
        None => DEFAULT_RETRY_POLICY.to_owned(),
    };
    let now = unix_now()?;
    let id = Uuid::new_v4().to_string();

    let mut tx = pool
        .begin()
        .await
        .context("could not start schedule transaction")?;
    let inserted = sqlx::query(
        "INSERT INTO schedules \
         (id, name, program, args_json, timing_json, parameters_json, missed_policy, \
          parameter_update_trigger_json, retry_policy_json, idempotency_key, idempotency_fingerprint, \
          created_at, updated_at, last_evaluated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&schedule.name)
    .bind(&schedule.program)
    .bind(args_json)
    .bind(timing_json)
    .bind(parameters_json)
    .bind(&schedule.missed_policy)
    .bind(trigger_json)
    .bind(retry_json)
    .bind(&schedule.idempotency_key)
    .bind(schedule.idempotency_key.as_ref().map(|_| fingerprint.as_str()))
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await;

    if let Err(error) = inserted {
        tx.rollback()
            .await
            .context("could not roll back failed schedule creation")?;
        if let Some(key) = &schedule.idempotency_key {
            let previous = sqlx::query_as::<_, (String, String)>(
                "SELECT id, idempotency_fingerprint FROM schedules WHERE idempotency_key = ?",
            )
            .bind(key)
            .fetch_optional(pool)
            .await
            .context("could not check prior idempotent schedule")?;
            if let Some((previous_id, previous_fingerprint)) = previous {
                if previous_fingerprint == fingerprint {
                    return Ok(CreateScheduleOutcome::Replayed { id: previous_id });
                }
                bail!("idempotency key was already used with different request content");
            }
        }
        return Err(error).context("could not insert schedule");
    }

    tx.commit()
        .await
        .context("could not commit schedule creation")?;
    Ok(CreateScheduleOutcome::Created { id })
}

pub async fn update_parameters(
    pool: &SqlitePool,
    schedule_id: &str,
    parameters: &Value,
) -> Result<bool> {
    let parameters_json = serialize_parameters(parameters)?;
    let result =
        sqlx::query("UPDATE schedules SET parameters_json = ?, updated_at = ? WHERE id = ?")
            .bind(parameters_json)
            .bind(unix_now()?)
            .bind(schedule_id)
            .execute(pool)
            .await
            .context("could not update schedule parameters")?;
    Ok(result.rows_affected() == 1)
}

/// Advances the recovery cursor only if it still matches the value previously read.
/// A false result means another scheduler pass already advanced the schedule.
pub async fn advance_evaluation_cursor(
    pool: &SqlitePool,
    schedule_id: &str,
    expected_cursor: i64,
    new_cursor: i64,
) -> Result<bool> {
    anyhow::ensure!(
        new_cursor >= expected_cursor,
        "evaluation cursor cannot move backwards"
    );
    let result = sqlx::query(
        "UPDATE schedules SET last_evaluated_at = ?, updated_at = ? \
         WHERE id = ? AND last_evaluated_at = ? AND status = 'active'",
    )
    .bind(new_cursor)
    .bind(unix_now()?)
    .bind(schedule_id)
    .bind(expected_cursor)
    .execute(pool)
    .await
    .context("could not advance schedule evaluation cursor")?;
    Ok(result.rows_affected() == 1)
}

/// Persists recovered queued occurrences and advances the cursor in one transaction.
pub async fn persist_recovery_plan(
    pool: &SqlitePool,
    schedule_id: &str,
    expected_cursor: i64,
    plan: &RecoveryPlan,
) -> Result<bool> {
    anyhow::ensure!(
        plan.new_cursor >= expected_cursor,
        "evaluation cursor cannot move backwards"
    );
    let now = unix_now()?;
    let mut tx = pool
        .begin()
        .await
        .context("could not start recovery transaction")?;
    let advanced = sqlx::query(
        "UPDATE schedules SET last_evaluated_at = ?, updated_at = ? \
         WHERE id = ? AND last_evaluated_at = ? AND status = 'active'",
    )
    .bind(plan.new_cursor)
    .bind(now)
    .bind(schedule_id)
    .bind(expected_cursor)
    .execute(&mut *tx)
    .await
    .context("could not advance schedule recovery cursor")?;
    if advanced.rows_affected() == 0 {
        tx.rollback()
            .await
            .context("could not roll back stale recovery transaction")?;
        return Ok(false);
    }

    for scheduled_for in &plan.individual {
        insert_recovery_occurrence(&mut tx, schedule_id, *scheduled_for, None).await?;
    }
    if let Some(range) = plan.consolidated {
        insert_recovery_occurrence(&mut tx, schedule_id, range.through, Some(range)).await?;
    }
    tx.commit()
        .await
        .context("could not commit recovery transaction")?;
    Ok(true)
}

async fn insert_recovery_occurrence(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    schedule_id: &str,
    scheduled_for: i64,
    missed: Option<crate::recurrence::MissedRange>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO occurrences (id, schedule_id, status, source, scheduled_for, created_at, \
         missed_count, missed_from, missed_through) VALUES (?, ?, 'queued', 'recovery', ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(schedule_id)
    .bind(scheduled_for)
    .bind(unix_now()?)
    .bind(missed.map(|value| value.count as i64))
    .bind(missed.map(|value| value.from))
    .bind(missed.map(|value| value.through))
    .execute(&mut **tx)
    .await
    .context("could not persist recovered occurrence")?;
    Ok(())
}

pub async fn prune_expired_history(pool: &SqlitePool, now: i64) -> Result<u64> {
    let cutoff = now.saturating_sub(HISTORY_RETENTION_MICROSECONDS);
    let result =
        sqlx::query("DELETE FROM occurrences WHERE finished_at IS NOT NULL AND finished_at < ?")
            .bind(cutoff)
            .execute(pool)
            .await
            .context("could not remove expired execution history")?;
    Ok(result.rows_affected())
}

pub async fn start_retention_worker(pool: SqlitePool) -> Result<()> {
    prune_expired_history(&pool, unix_now()?)
        .await
        .context("could not apply startup execution-history retention")?;
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
            let now = match unix_now() {
                Ok(now) => now,
                Err(error) => {
                    eprintln!("execution-history retention skipped: {error:#}");
                    continue;
                }
            };
            if let Err(error) = prune_expired_history(&pool, now).await {
                eprintln!("execution-history retention failed: {error:#}");
            }
        }
    });
    Ok(())
}

fn validate_schedule(schedule: &NewSchedule) -> Result<()> {
    if schedule.program.trim().is_empty() {
        bail!("program must not be empty");
    }
    if !matches!(
        schedule.missed_policy.as_str(),
        "skip" | "consolidate" | "sequence"
    ) {
        bail!("missed policy must be skip, consolidate, or sequence");
    }
    if let Some(key) = &schedule.idempotency_key {
        if key.is_empty() {
            bail!("idempotency key must not be empty");
        }
    }
    serialize_parameters(schedule.parameters.as_ref().unwrap_or(&Value::Null))?;
    for (field, value) in [
        ("timing", Some(&schedule.timing)),
        (
            "parameter update trigger",
            schedule.parameter_update_trigger.as_ref(),
        ),
        ("retry policy", schedule.retry_policy.as_ref()),
    ] {
        if let Some(value) = value {
            if !value.is_object() {
                bail!("{field} must be a JSON object");
            }
        }
    }
    Ok(())
}

fn serialize_json(value: &Value) -> Result<String> {
    serde_json::to_string(value).context("could not encode JSON value")
}

fn serialize_parameters(value: &Value) -> Result<String> {
    let parameters = serde_json::to_string(value).context("could not encode parameters")?;
    if parameters.len() > MAX_PARAMETERS_BYTES {
        bail!("parameters exceed the 64 KiB limit");
    }
    Ok(parameters)
}

fn hex_digest(content: &[u8]) -> String {
    Sha256::digest(content)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn unix_now() -> Result<i64> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")?;
    i64::try_from(elapsed.as_micros()).context("system clock timestamp is out of range")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_schedule(key: Option<&str>) -> NewSchedule {
        NewSchedule {
            name: Some("daily job".to_owned()),
            program: "/usr/bin/python3".to_owned(),
            args: vec!["job.py".to_owned()],
            timing: serde_json::json!({"type": "cron", "expression": "30 8 * * 1-5"}),
            parameters: Some(serde_json::json!("arbitrary JSON scalar")),
            missed_policy: "consolidate".to_owned(),
            parameter_update_trigger: None,
            retry_policy: None,
            idempotency_key: key.map(str::to_owned),
        }
    }

    async fn test_database() -> (tempfile::TempDir, SqlitePool) {
        let directory = tempfile::tempdir().expect("temp directory");
        let database = connect_at(&directory.path().join("scheduler.sqlite3"))
            .await
            .expect("database and migrations");
        (directory, database)
    }

    #[tokio::test]
    async fn migrations_and_schedule_data_survive_reopen() {
        let (directory, database) = test_database().await;
        let created = create_schedule(&database, &sample_schedule(None))
            .await
            .expect("create schedule");
        let CreateScheduleOutcome::Created { id } = created else {
            panic!("expected new schedule")
        };
        database.close().await;

        let reopened = connect_at(&directory.path().join("scheduler.sqlite3"))
            .await
            .expect("reopen and migrate");
        let saved: (String, String, String) = sqlx::query_as(
            "SELECT program, args_json, parameters_json FROM schedules WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&reopened)
        .await
        .expect("persisted row");
        assert_eq!(saved.0, "/usr/bin/python3");
        assert_eq!(
            serde_json::from_str::<Value>(&saved.1).unwrap(),
            serde_json::json!(["job.py"])
        );
        assert_eq!(
            serde_json::from_str::<Value>(&saved.2).unwrap(),
            serde_json::json!("arbitrary JSON scalar")
        );
    }

    #[tokio::test]
    async fn idempotent_create_replays_same_request_and_rejects_conflict() {
        let (_directory, database) = test_database().await;
        let first = create_schedule(&database, &sample_schedule(Some("request-1")))
            .await
            .unwrap();
        let replay = create_schedule(&database, &sample_schedule(Some("request-1")))
            .await
            .unwrap();
        assert!(
            matches!((&first, &replay), (CreateScheduleOutcome::Created { id: a }, CreateScheduleOutcome::Replayed { id: b }) if a == b)
        );

        let mut different = sample_schedule(Some("request-1"));
        different.args.push("different".to_owned());
        let error = create_schedule(&database, &different).await.unwrap_err();
        assert!(error.to_string().contains("different request content"));

        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM schedules")
            .fetch_one(&database)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn parameters_accept_json_values_and_reject_more_than_64_kib() {
        let (_directory, database) = test_database().await;
        let created = create_schedule(&database, &sample_schedule(None))
            .await
            .unwrap();
        let CreateScheduleOutcome::Created { id } = created else {
            panic!("expected new schedule")
        };
        assert!(
            update_parameters(&database, &id, &serde_json::json!(null))
                .await
                .unwrap()
        );
        assert!(
            update_parameters(&database, &id, &serde_json::json!([true, 1, "x"]))
                .await
                .unwrap()
        );

        let oversized = Value::String("x".repeat(MAX_PARAMETERS_BYTES));
        let error = update_parameters(&database, &id, &oversized)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("64 KiB"));
        let saved: String =
            sqlx::query_scalar("SELECT parameters_json FROM schedules WHERE id = ?")
                .bind(id)
                .fetch_one(&database)
                .await
                .unwrap();
        assert_eq!(saved, "[true,1,\"x\"]");
    }

    #[tokio::test]
    async fn retention_removes_old_occurrences_and_keeps_schedule() {
        let (_directory, database) = test_database().await;
        let created = create_schedule(&database, &sample_schedule(None))
            .await
            .unwrap();
        let CreateScheduleOutcome::Created { id } = created else {
            panic!("expected new schedule")
        };
        let now = 2_000_000_000_000_000_i64;
        for (occurrence_id, finished_at) in [
            ("old", now - HISTORY_RETENTION_MICROSECONDS - 1),
            ("new", now),
        ] {
            sqlx::query("INSERT INTO occurrences (id, schedule_id, status, source, created_at, finished_at) VALUES (?, ?, 'succeeded', 'schedule', ?, ?)")
                .bind(occurrence_id)
                .bind(&id)
                .bind(now - 10)
                .bind(finished_at)
                .execute(&database)
                .await
                .unwrap();
        }
        assert_eq!(prune_expired_history(&database, now).await.unwrap(), 1);

        let schedules: i64 = sqlx::query_scalar("SELECT count(*) FROM schedules WHERE id = ?")
            .bind(id)
            .fetch_one(&database)
            .await
            .unwrap();
        let occurrences: i64 = sqlx::query_scalar("SELECT count(*) FROM occurrences")
            .fetch_one(&database)
            .await
            .unwrap();
        assert_eq!(schedules, 1);
        assert_eq!(occurrences, 1);
    }

    #[tokio::test]
    async fn recovery_occurrences_and_cursor_commit_atomically() {
        let (_directory, database) = test_database().await;
        let created = create_schedule(&database, &sample_schedule(None))
            .await
            .unwrap();
        let CreateScheduleOutcome::Created { id } = created else {
            panic!("expected newly created schedule")
        };
        let cursor: i64 =
            sqlx::query_scalar("SELECT last_evaluated_at FROM schedules WHERE id = ?")
                .bind(&id)
                .fetch_one(&database)
                .await
                .unwrap();
        let plan = RecoveryPlan {
            individual: vec![cursor + 10, cursor + 20],
            consolidated: Some(crate::recurrence::MissedRange {
                count: 3,
                from: cursor + 30,
                through: cursor + 50,
            }),
            new_cursor: cursor + 60,
        };
        assert!(
            persist_recovery_plan(&database, &id, cursor, &plan)
                .await
                .unwrap()
        );
        assert!(
            !persist_recovery_plan(&database, &id, cursor, &plan)
                .await
                .unwrap()
        );

        let counts: (i64, i64) = sqlx::query_as(
            "SELECT count(*), count(missed_count) FROM occurrences WHERE schedule_id = ?",
        )
        .bind(&id)
        .fetch_one(&database)
        .await
        .unwrap();
        assert_eq!(counts, (3, 1));
        let saved_cursor: i64 =
            sqlx::query_scalar("SELECT last_evaluated_at FROM schedules WHERE id = ?")
                .bind(id)
                .fetch_one(&database)
                .await
                .unwrap();
        assert_eq!(saved_cursor, plan.new_cursor);
    }
}
