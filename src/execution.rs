use std::{
    collections::HashMap,
    process::Stdio,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

use crate::database::{self as sqlx, SqlitePool};
use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    sync::oneshot,
    task::JoinHandle,
};
use uuid::Uuid;

use crate::process_tree::ProcessTree;

const MAX_CAPTURE_BYTES: usize = 1024 * 1024;
pub const COMMAND_TEST_TIMEOUT: Duration = Duration::from_secs(30);
static CANCELLATIONS: OnceLock<Mutex<HashMap<String, oneshot::Sender<()>>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionOutcome {
    NotQueued,
    Cancelled,
    Finished {
        succeeded: bool,
        exit_code: Option<i32>,
        retry_id: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RetryPolicy {
    #[serde(default = "default_max_retries")]
    max_retries: u64,
    #[serde(default = "default_retry_delays")]
    delays_seconds: Vec<u64>,
}

fn default_max_retries() -> u64 {
    3
}
fn default_retry_delays() -> Vec<u64> {
    vec![60, 300, 900]
}

#[derive(Clone, Default)]
struct CapturedOutput {
    bytes: Vec<u8>,
    truncated: bool,
}

pub struct CommandTestResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub timed_out: bool,
}

pub async fn test_command(
    program: &str,
    args: &[String],
    parameters: &serde_json::Value,
) -> Result<CommandTestResult> {
    let test_deadline = tokio::time::Instant::now() + COMMAND_TEST_TIMEOUT;
    let mut command = Command::new(program);
    command
        .args(args)
        .env(
            "COMMAND_SCHEDULER_PARAMS_JSON",
            serde_json::to_string(parameters)?,
        )
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let (child, tree) = match ProcessTree::spawn(&mut command) {
        Ok(process) => process,
        Err(error) => {
            return Ok(CommandTestResult {
                exit_code: None,
                stdout: String::new(),
                stderr: format!("could not start {program}: {error}"),
                stdout_truncated: false,
                stderr_truncated: false,
                timed_out: false,
            });
        }
    };
    let result = run_child(child, tree, Some(test_deadline), None).await?;
    Ok(CommandTestResult {
        exit_code: result.status.code(),
        stdout: String::from_utf8_lossy(&result.stdout.bytes).into_owned(),
        stderr: String::from_utf8_lossy(&result.stderr.bytes).into_owned(),
        stdout_truncated: result.stdout.truncated,
        stderr_truncated: result.stderr.truncated,
        timed_out: result.timed_out,
    })
}

#[allow(clippy::type_complexity)]
pub async fn execute_queued(pool: &SqlitePool, occurrence_id: &str) -> Result<ExecutionOutcome> {
    let (cancel_sender, cancel_receiver) = oneshot::channel();
    cancellation_map()
        .lock()
        .expect("cancellation registry poisoned")
        .insert(occurrence_id.to_owned(), cancel_sender);
    let mut tx = pool
        .begin()
        .await
        .context("could not start occurrence claim")?;
    let claimed = sqlx::query(
        "UPDATE occurrences SET status = 'running', started_at = ? \
         WHERE id = ? AND status = 'queued' AND (source != 'retry' OR EXISTS (\
           SELECT 1 FROM schedules s WHERE s.id=occurrences.schedule_id AND s.status!='cancelled'))",
    )
    .bind(now_micros()?)
    .bind(occurrence_id)
    .execute(&mut *tx)
    .await
    .context("could not claim queued occurrence")?;
    if claimed.rows_affected() == 0 {
        cancellation_map()
            .lock()
            .expect("cancellation registry poisoned")
            .remove(occurrence_id);
        tx.rollback()
            .await
            .context("could not roll back unclaimed occurrence")?;
        return Ok(ExecutionOutcome::NotQueued);
    }

    let row = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            i64,
            String,
        ),
    >(
        "SELECT s.program, s.args_json, s.parameters_json, o.missed_count, o.missed_from, \
         o.missed_through, o.attempt, s.retry_policy_json \
         FROM occurrences o JOIN schedules s ON s.id = o.schedule_id WHERE o.id = ?",
    )
    .bind(occurrence_id)
    .fetch_one(&mut *tx)
    .await
    .context("could not load occurrence execution context")?;
    tx.commit()
        .await
        .context("could not commit occurrence claim")?;

    let (
        program,
        args_json,
        parameters_json,
        missed_count,
        missed_from,
        missed_through,
        attempt,
        retry_policy_json,
    ) = row;
    let args: Vec<String> =
        serde_json::from_str(&args_json).context("stored command arguments are invalid")?;
    let parameters: serde_json::Value =
        serde_json::from_str(&parameters_json).context("stored parameters are invalid")?;
    let retry_policy: RetryPolicy =
        serde_json::from_str(&retry_policy_json).unwrap_or(RetryPolicy {
            max_retries: 0,
            delays_seconds: Vec::new(),
        });

    let mut command = Command::new(&program);
    command
        .args(args)
        .env(
            "COMMAND_SCHEDULER_PARAMS_JSON",
            serde_json::to_string(&parameters)?,
        )
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if let (Some(count), Some(from), Some(through)) = (missed_count, missed_from, missed_through) {
        command
            .env("COMMAND_SCHEDULER_MISSED_COUNT", count.to_string())
            .env("COMMAND_SCHEDULER_MISSED_FROM", rfc3339(from)?)
            .env("COMMAND_SCHEDULER_MISSED_THROUGH", rfc3339(through)?);
    }

    let process_result = ProcessTree::spawn(&mut command);
    let (exit_code, success, cancelled, stdout, stderr) = match process_result {
        Ok((child, tree)) => {
            let mut result = run_child(child, tree, None, Some(cancel_receiver)).await?;
            if result.timed_out {
                result
                    .stderr
                    .append(b"\nTimed out draining descendant process output.\n");
            }
            (
                result.status.code(),
                result.status.success() && !result.timed_out,
                result.cancelled,
                result.stdout,
                result.stderr,
            )
        }
        Err(error) => (
            None,
            false,
            false,
            CapturedOutput {
                bytes: Vec::new(),
                truncated: false,
            },
            CapturedOutput {
                bytes: format!("could not start {program}: {error}\n").into_bytes(),
                truncated: false,
            },
        ),
    };
    cancellation_map()
        .lock()
        .expect("cancellation registry poisoned")
        .remove(occurrence_id);

    let finished_at = now_micros()?;
    let status = if cancelled {
        "cancelled"
    } else if success {
        "succeeded"
    } else {
        "failed"
    };
    let mut tx = pool
        .begin()
        .await
        .context("could not start occurrence completion")?;
    sqlx::query(
        "UPDATE occurrences SET status = ?, finished_at = ?, exit_code = ?, stdout = ?, stderr = ?, \
         stdout_truncated = ?, stderr_truncated = ? WHERE id = ? AND status = 'running'",
    )
    .bind(status)
    .bind(finished_at)
    .bind(exit_code)
    .bind(stdout.bytes)
    .bind(stderr.bytes)
    .bind(i64::from(stdout.truncated))
    .bind(i64::from(stderr.truncated))
    .bind(occurrence_id)
    .execute(&mut *tx)
    .await
    .context("could not persist occurrence result")?;

    let retry_id = if cancelled
        || success
        || u64::try_from(attempt).unwrap_or(u64::MAX) > retry_policy.max_retries
    {
        None
    } else if let Some(delay) = retry_policy
        .delays_seconds
        .get(usize::try_from(attempt.saturating_sub(1)).unwrap_or(usize::MAX))
        .or_else(|| retry_policy.delays_seconds.last())
    {
        let retry_id = Uuid::new_v4().to_string();
        let delay = i64::try_from(*delay).unwrap_or(i64::MAX);
        let scheduled_for = finished_at.saturating_add(delay.saturating_mul(1_000_000));
        let inserted = sqlx::query(
            "INSERT INTO occurrences (id, schedule_id, status, source, attempt, retry_of_id, \
             scheduled_for, created_at, missed_count, missed_from, missed_through) \
             SELECT ?, o.schedule_id, 'queued', 'retry', ?, o.id, ?, ?, o.missed_count, o.missed_from, o.missed_through \
             FROM occurrences o JOIN schedules s ON s.id=o.schedule_id \
             WHERE o.id = ? AND s.status != 'cancelled'",
        )
        .bind(&retry_id)
        .bind(attempt.saturating_add(1))
        .bind(scheduled_for)
        .bind(finished_at)
        .bind(occurrence_id)
        .execute(&mut *tx)
        .await
        .context("could not persist automatic retry")?;
        (inserted.rows_affected() > 0).then_some(retry_id)
    } else {
        None
    };
    tx.commit()
        .await
        .context("could not commit occurrence result")?;

    if cancelled {
        return Ok(ExecutionOutcome::Cancelled);
    }
    Ok(ExecutionOutcome::Finished {
        succeeded: success,
        exit_code,
        retry_id,
    })
}

pub fn request_cancel(occurrence_id: String) -> bool {
    let sender = cancellation_map()
        .lock()
        .expect("cancellation registry poisoned")
        .remove(&occurrence_id);
    sender.is_some_and(|sender| sender.send(()).is_ok())
}

fn cancellation_map() -> &'static Mutex<HashMap<String, oneshot::Sender<()>>> {
    CANCELLATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

impl CapturedOutput {
    fn append(&mut self, buffer: &[u8]) {
        let retained = buffer
            .len()
            .min(MAX_CAPTURE_BYTES.saturating_sub(self.bytes.len()));
        self.bytes.extend_from_slice(&buffer[..retained]);
        self.truncated |= retained < buffer.len();
    }
}

struct CaptureTask {
    task: JoinHandle<Result<()>>,
    output: Arc<Mutex<CapturedOutput>>,
}

impl CaptureTask {
    fn start<R: AsyncRead + Unpin + Send + 'static>(reader: R) -> Self {
        let output = Arc::new(Mutex::new(CapturedOutput::default()));
        let task = tokio::spawn(capture_limited(reader, output.clone()));
        Self { task, output }
    }

    fn snapshot(&self) -> CapturedOutput {
        let mut output = self.output.lock().expect("capture buffer poisoned").clone();
        output.truncated |= !self.task.is_finished();
        output
    }
}

impl Drop for CaptureTask {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn capture_limited<R: AsyncRead + Unpin>(
    mut reader: R,
    output: Arc<Mutex<CapturedOutput>>,
) -> Result<()> {
    let mut buffer = [0u8; 8192];
    loop {
        let read = reader
            .read(&mut buffer)
            .await
            .context("could not read child output")?;
        if read == 0 {
            return Ok(());
        }
        output
            .lock()
            .expect("capture buffer poisoned")
            .append(&buffer[..read]);
    }
}

struct ChildResult {
    status: std::process::ExitStatus,
    stdout: CapturedOutput,
    stderr: CapturedOutput,
    cancelled: bool,
    timed_out: bool,
}

async fn cancellation(receiver: &mut Option<oneshot::Receiver<()>>) {
    if let Some(receiver) = receiver {
        let _ = receiver.await;
    } else {
        std::future::pending::<()>().await;
    }
}

async fn deadline(at: Option<tokio::time::Instant>) {
    if let Some(at) = at {
        tokio::time::sleep_until(at).await;
    } else {
        std::future::pending::<()>().await;
    }
}

async fn run_child(
    mut child: tokio::process::Child,
    tree: ProcessTree,
    total_deadline: Option<tokio::time::Instant>,
    mut cancel_receiver: Option<oneshot::Receiver<()>>,
) -> Result<ChildResult> {
    let mut stdout = CaptureTask::start(
        child
            .stdout
            .take()
            .context("child stdout pipe is missing")?,
    );
    let mut stderr = CaptureTask::start(
        child
            .stderr
            .take()
            .context("child stderr pipe is missing")?,
    );
    let mut cancelled = false;
    let mut timed_out = false;
    let status = tokio::select! {
        result = child.wait() => result.context("could not wait for child process")?,
        _ = cancellation(&mut cancel_receiver) => {
            cancelled = true;
            cancel_receiver = None;
            tree.terminate();
            child.start_kill().context("could not terminate child process")?;
            child.wait().await.context("could not reap cancelled child")?
        }
        _ = deadline(total_deadline) => {
            timed_out = true;
            tree.terminate();
            child.start_kill().context("could not terminate timed out child")?;
            child.wait().await.context("could not reap timed out child")?
        }
    };
    // Tests share one deadline across process and pipes. Scheduled commands have
    // no runtime deadline, but draining after the parent exits is bounded.
    let drain_deadline = if cancelled || timed_out {
        tokio::time::Instant::now() + Duration::from_millis(100)
    } else {
        total_deadline.unwrap_or_else(|| tokio::time::Instant::now() + COMMAND_TEST_TIMEOUT)
    };
    tokio::select! {
        result = async { tokio::try_join!(&mut stdout.task, &mut stderr.task) } => {
            let (out, err) = result.context("output capture task failed")?;
            out?;
            err?;
        }
        _ = cancellation(&mut cancel_receiver) => {
            cancelled = true;
            cancel_receiver = None;
            tree.terminate();
        }
        _ = tokio::time::sleep_until(drain_deadline) => {
            timed_out = true;
            tree.terminate();
        }
    }
    // A cancellation accepted while both pipes finish still belongs to this run.
    if cancel_receiver
        .as_mut()
        .is_some_and(|receiver| receiver.try_recv().is_ok())
    {
        cancelled = true;
        tree.terminate();
    }
    Ok(ChildResult {
        status,
        stdout: stdout.snapshot(),
        stderr: stderr.snapshot(),
        cancelled,
        timed_out,
    })
}

fn rfc3339(timestamp: i64) -> Result<String> {
    DateTime::<Utc>::from_timestamp_micros(timestamp)
        .map(|value| value.to_rfc3339())
        .ok_or_else(|| anyhow!("missed occurrence timestamp is out of range"))
}

fn now_micros() -> Result<i64> {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("system clock is before Unix epoch")?;
    i64::try_from(duration.as_micros()).context("system clock timestamp is out of range")
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::time::Duration;

    use super::*;
    #[cfg(unix)]
    use crate::storage::{CreateScheduleOutcome, NewSchedule, connect_at, create_schedule};
    #[cfg(unix)]
    use serde_json::{Value, json};

    #[tokio::test]
    async fn output_capture_retains_one_mib_and_drains_the_rest() {
        let output = vec![b'x'; MAX_CAPTURE_BYTES + 17];
        let captured = Arc::new(Mutex::new(CapturedOutput::default()));
        capture_limited(output.as_slice(), captured.clone())
            .await
            .unwrap();
        let captured = captured.lock().unwrap();
        assert_eq!(captured.bytes.len(), MAX_CAPTURE_BYTES);
        assert!(captured.truncated);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn deadline_terminates_a_running_parent_and_preserves_output() {
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", "printf retained; sleep 5"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let (child, tree) = ProcessTree::spawn(&mut command).unwrap();
        let start = tokio::time::Instant::now();
        let result = run_child(child, tree, Some(start + Duration::from_millis(250)), None)
            .await
            .unwrap();
        assert!(result.timed_out);
        assert!(!result.status.success());
        assert_eq!(result.stdout.bytes, b"retained");
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn process_and_descendant_pipes_share_one_deadline() {
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", "printf retained; sleep 5 & sleep 0.05"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let (child, tree) = ProcessTree::spawn(&mut command).unwrap();
        let start = tokio::time::Instant::now();
        let result = run_child(child, tree, Some(start + Duration::from_millis(350)), None)
            .await
            .unwrap();
        assert!(result.status.success());
        assert!(result.timed_out);
        assert_eq!(result.stdout.bytes, b"retained");
        assert!(result.stdout.truncated);
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[cfg(unix)]
    async fn occurrence(
        program: &str,
        args: &[&str],
        retry: Option<Value>,
    ) -> (tempfile::TempDir, SqlitePool, String) {
        let dir = tempfile::tempdir().unwrap();
        let pool = connect_at(&dir.path().join("test.sqlite3")).await.unwrap();
        let schedule = NewSchedule {
            name: None,
            program: program.to_owned(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            timing: json!({"type":"cron", "expression":"* * * * *"}),
            parameters: Some(json!({"quoted": "value with spaces"})),
            missed_policy: "skip".to_owned(),
            parameter_update_trigger: None,
            retry_policy: retry,
            idempotency_key: None,
        };
        let CreateScheduleOutcome::Created { id: schedule_id } =
            create_schedule(&pool, &schedule).await.unwrap()
        else {
            panic!("schedule created")
        };
        let occurrence_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO occurrences (id,schedule_id,status,source,created_at) VALUES (?,?,'queued','manual',?)")
            .bind(&occurrence_id).bind(schedule_id).bind(now_micros().unwrap()).execute(&pool).await.unwrap();
        (dir, pool, occurrence_id)
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn invokes_program_with_literal_arguments_and_captures_both_streams() {
        let (_dir, pool, id) = occurrence(
            "/bin/sh",
            &["-c", "printf '%s' '$HOME'; printf 'warning' >&2"],
            None,
        )
        .await;
        let outcome = execute_queued(&pool, &id).await.unwrap();
        assert_eq!(
            outcome,
            ExecutionOutcome::Finished {
                succeeded: true,
                exit_code: Some(0),
                retry_id: None
            }
        );
        let saved: (Vec<u8>, Vec<u8>, String) =
            sqlx::query_as("SELECT stdout, stderr, status FROM occurrences WHERE id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(saved.0, b"$HOME");
        assert_eq!(saved.1, b"warning");
        assert_eq!(saved.2, "succeeded");
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn failed_execution_persists_a_delayed_retry() {
        let (_dir, pool, id) = occurrence(
            "/bin/sh",
            &["-c", "exit 7"],
            Some(serde_json::json!({"maxRetries": 3, "delaysSeconds": [1,5,15]})),
        )
        .await;
        let before = now_micros().unwrap();
        let outcome = execute_queued(&pool, &id).await.unwrap();
        let ExecutionOutcome::Finished {
            succeeded: false,
            exit_code: Some(7),
            retry_id: Some(retry_id),
        } = outcome
        else {
            panic!("failed command should be retried")
        };
        let retry: (String, i64, i64, String) = sqlx::query_as(
            "SELECT status, attempt, scheduled_for, source FROM occurrences WHERE id = ?",
        )
        .bind(retry_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            (retry.0.as_str(), retry.1, retry.3.as_str()),
            ("queued", 2, "retry")
        );
        assert!(retry.2 >= before + 1_000_000);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn cancellation_terminates_the_child_without_retry() {
        let (_dir, pool, id) = occurrence(
            "/bin/sleep",
            &["30"],
            Some(json!({"maxRetries": 3, "delaysSeconds": [1,5,15]})),
        )
        .await;
        let running_id = id.clone();
        let running_pool = pool.clone();
        let task =
            tokio::spawn(async move { execute_queued(&running_pool, &running_id).await.unwrap() });
        for _ in 0..100 {
            let status: Option<String> =
                sqlx::query_scalar("SELECT status FROM occurrences WHERE id=?")
                    .bind(&id)
                    .fetch_optional(&pool)
                    .await
                    .unwrap();
            if status.as_deref() == Some("running") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert!(request_cancel(id.clone()));
        assert_eq!(task.await.unwrap(), ExecutionOutcome::Cancelled);
        let saved:(String,i64)=sqlx::query_as("SELECT status, count(*) FROM occurrences WHERE schedule_id=(SELECT schedule_id FROM occurrences WHERE id=?)").bind(id).fetch_one(&pool).await.unwrap();
        assert_eq!(saved.0, "cancelled");
        assert_eq!(saved.1, 1);
    }
}
