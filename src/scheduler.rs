use std::{collections::HashSet, time::Duration};

use crate::database::SqlitePool;
use anyhow::{Context, Result, bail};
use chrono::DateTime;
use serde_json::Value;
use tokio::task::JoinSet;

use crate::{
    execution::{ExecutionOutcome, execute_queued},
    recurrence::{MissedPolicy, next_local_occurrence, recover_missed},
    storage::{
        ActiveSchedule, active_schedules, due_occurrences, mark_running_occurrences_interrupted,
        persist_recovery_plan, persist_scheduled_occurrence,
    },
};

const POLL_INTERVAL: Duration = Duration::from_millis(250);
const ON_TIME_WINDOW_MICROSECONDS: i64 = 5_000_000;

pub async fn recover_interrupted(pool: &SqlitePool) -> Result<u64> {
    mark_running_occurrences_interrupted(pool, now_micros()?)
        .await
        .context("could not recover processes left running by previous daemon")
}

pub fn start(pool: SqlitePool) {
    tokio::spawn(async move {
        run(pool).await;
    });
}

async fn run(pool: SqlitePool) {
    let mut executions = JoinSet::new();
    let mut active = HashSet::new();
    loop {
        while let Some(result) = executions.try_join_next() {
            match result {
                Ok((id, Ok(outcome))) => {
                    active.remove(&id);
                    if let ExecutionOutcome::Finished {
                        succeeded,
                        exit_code,
                        retry_id,
                    } = outcome
                    {
                        println!(
                            "occurrence {id} finished: succeeded={succeeded} exit_code={exit_code:?} retry={retry_id:?}"
                        );
                    }
                }
                Ok((id, Err(error))) => {
                    active.remove(&id);
                    eprintln!("occurrence {id} execution failed: {error:#}");
                }
                Err(error) => eprintln!("occurrence worker failed: {error}"),
            }
        }

        let now = match now_micros() {
            Ok(now) => now,
            Err(error) => {
                eprintln!("scheduler clock unavailable: {error:#}");
                tokio::time::sleep(POLL_INTERVAL).await;
                continue;
            }
        };
        match tick_schedules(&pool, now).await {
            Ok(()) => {}
            Err(error) => eprintln!("scheduler pass failed: {error:#}"),
        }
        match due_occurrences(&pool, now).await {
            Ok(due) => {
                for id in due {
                    if active.insert(id.clone()) {
                        let pool = pool.clone();
                        executions.spawn(async move {
                            let result = execute_queued(&pool, &id).await;
                            (id, result)
                        });
                    }
                }
            }
            Err(error) => eprintln!("could not query due occurrences: {error:#}"),
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn tick_schedules(pool: &SqlitePool, now: i64) -> Result<()> {
    for schedule in active_schedules(pool).await? {
        if let Err(error) = evaluate_schedule(pool, &schedule, now).await {
            eprintln!("could not evaluate schedule {}: {error:#}", schedule.id);
        }
    }
    Ok(())
}

async fn evaluate_schedule(pool: &SqlitePool, schedule: &ActiveSchedule, now: i64) -> Result<()> {
    let candidate = next_timing_occurrence(&schedule.timing, schedule.last_evaluated_at)?;
    let Some(scheduled_for) = candidate.filter(|candidate| *candidate <= now) else {
        return Ok(());
    };

    if now.saturating_sub(scheduled_for) <= ON_TIME_WINDOW_MICROSECONDS {
        persist_scheduled_occurrence(
            pool,
            &schedule.id,
            schedule.last_evaluated_at,
            scheduled_for,
        )
        .await?;
        return Ok(());
    }

    let policy = parse_policy(&schedule.missed_policy)?;
    let plan = match schedule.timing.get("type").and_then(Value::as_str) {
        Some("cron") => {
            let expression = schedule
                .timing
                .get("expression")
                .and_then(Value::as_str)
                .context("cron timing has no expression")?;
            recover_missed(expression, schedule.last_evaluated_at, now, policy)?
        }
        Some("once") => {
            if policy == MissedPolicy::Skip {
                crate::recurrence::RecoveryPlan {
                    individual: Vec::new(),
                    consolidated: None,
                    new_cursor: now,
                }
            } else {
                crate::recurrence::RecoveryPlan {
                    individual: vec![scheduled_for],
                    consolidated: None,
                    new_cursor: now,
                }
            }
        }
        Some(other) => bail!("unsupported timing type: {other}"),
        None => bail!("timing has no type"),
    };
    persist_recovery_plan(pool, &schedule.id, schedule.last_evaluated_at, &plan).await?;
    Ok(())
}

fn next_timing_occurrence(timing: &Value, after: i64) -> Result<Option<i64>> {
    match timing.get("type").and_then(Value::as_str) {
        Some("cron") => {
            let expression = timing
                .get("expression")
                .and_then(Value::as_str)
                .context("cron timing has no expression")?;
            Ok(Some(next_local_occurrence(expression, after)?))
        }
        Some("once") => {
            let at = timing
                .get("at")
                .and_then(Value::as_str)
                .context("one-time timing has no timestamp")?;
            let timestamp = DateTime::parse_from_rfc3339(at)
                .context("one-time timestamp is not RFC 3339")?
                .timestamp_micros();
            Ok((timestamp > after).then_some(timestamp))
        }
        Some(other) => bail!("unsupported timing type: {other}"),
        None => bail!("timing has no type"),
    }
}

fn parse_policy(policy: &str) -> Result<MissedPolicy> {
    match policy {
        "skip" => Ok(MissedPolicy::Skip),
        "consolidate" => Ok(MissedPolicy::Consolidate),
        "sequence" => Ok(MissedPolicy::Sequence),
        _ => bail!("unknown missed occurrence policy: {policy}"),
    }
}

fn now_micros() -> Result<i64> {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("system clock is before Unix epoch")?;
    i64::try_from(duration.as_micros()).context("system clock timestamp is out of range")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database as sqlx;
    use crate::storage::{CreateScheduleOutcome, NewSchedule, connect_at, create_schedule};
    use chrono::Utc;
    use serde_json::json;

    #[tokio::test]
    async fn due_one_time_schedule_is_persisted_and_cursor_prevents_duplicates() {
        let directory = tempfile::tempdir().unwrap();
        let pool = connect_at(&directory.path().join("scheduler.sqlite3"))
            .await
            .unwrap();
        let due = now_micros().unwrap() - 20_000_000;
        let input = NewSchedule {
            name: None,
            program: "/bin/true".to_owned(),
            args: Vec::new(),
            timing: json!({"type":"once", "at": DateTime::<Utc>::from_timestamp_micros(due).unwrap().to_rfc3339()}),
            parameters: None,
            missed_policy: "sequence".to_owned(),
            parameter_update_trigger: None,
            retry_policy: None,
            idempotency_key: None,
        };
        let CreateScheduleOutcome::Created { id } = create_schedule(&pool, &input).await.unwrap()
        else {
            panic!("created")
        };
        sqlx::query("UPDATE schedules SET last_evaluated_at = ? WHERE id = ?")
            .bind(due - 1)
            .bind(&id)
            .execute(&pool)
            .await
            .unwrap();
        let schedule = active_schedules(&pool).await.unwrap().remove(0);
        evaluate_schedule(&pool, &schedule, now_micros().unwrap())
            .await
            .unwrap();
        evaluate_schedule(&pool, &schedule, now_micros().unwrap())
            .await
            .unwrap();
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM occurrences WHERE schedule_id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn resume_applies_saved_policy_to_times_missed_while_paused() {
        let directory = tempfile::tempdir().unwrap();
        let pool = connect_at(&directory.path().join("scheduler.sqlite3"))
            .await
            .unwrap();
        let due = now_micros().unwrap() - 20_000_000;
        let input = NewSchedule {
            name: None,
            program: "/bin/true".to_owned(),
            args: Vec::new(),
            timing: json!({"type":"once", "at": DateTime::<Utc>::from_timestamp_micros(due).unwrap().to_rfc3339()}),
            parameters: None,
            missed_policy: "sequence".to_owned(),
            parameter_update_trigger: None,
            retry_policy: None,
            idempotency_key: None,
        };
        let CreateScheduleOutcome::Created { id } = create_schedule(&pool, &input).await.unwrap()
        else {
            panic!("created")
        };
        sqlx::query("UPDATE schedules SET status = 'paused', last_evaluated_at = ? WHERE id = ?")
            .bind(due - 1)
            .bind(&id)
            .execute(&pool)
            .await
            .unwrap();

        let paused = active_schedules(&pool).await.unwrap();
        assert!(paused.is_empty());
        let cursor: i64 =
            sqlx::query_scalar("SELECT last_evaluated_at FROM schedules WHERE id = ?")
                .bind(&id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(cursor, due - 1);

        sqlx::query("UPDATE schedules SET status = 'active' WHERE id = ?")
            .bind(&id)
            .execute(&pool)
            .await
            .unwrap();
        let schedule = active_schedules(&pool).await.unwrap().remove(0);
        evaluate_schedule(&pool, &schedule, now_micros().unwrap())
            .await
            .unwrap();
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM occurrences WHERE schedule_id = ?")
                .bind(&id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1);
    }
}
