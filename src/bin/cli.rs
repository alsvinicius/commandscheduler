use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use std::{io::Write, net::IpAddr, process::ExitCode, time::Duration};

#[derive(Debug, Parser)]
#[command(version, about = "Command Scheduler local API client")]
struct Args {
    /// Daemon base URL (defaults to COMMAND_SCHEDULER_URL or http://127.0.0.1:30002)
    #[arg(
        long,
        global = true,
        env = "COMMAND_SCHEDULER_URL",
        default_value = "http://127.0.0.1:30002"
    )]
    server: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a one-time or recurring schedule
    Create {
        program: String,
        #[arg(long = "arg", action = clap::ArgAction::Append)]
        args: Vec<String>,
        #[arg(long, conflicts_with = "cron")]
        at: Option<String>,
        #[arg(long, conflicts_with = "at")]
        cron: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long, default_value = "null")]
        parameters: String,
        #[arg(long, default_value = "skip")]
        missed_policy: String,
        #[arg(long, default_value_t = 3)]
        max_retries: u32,
        #[arg(long, value_delimiter = ',', default_value = "60,300,900")]
        retry_delays: Vec<u64>,
        #[arg(long)]
        trigger_on_parameters: bool,
        #[arg(long, default_value = "coexist")]
        trigger_behavior: String,
    },
    /// List schedules
    Schedules,
    /// Inspect one schedule
    Show {
        id: String,
    },
    /// Update a schedule using a JSON object matching the API PATCH contract
    Update {
        id: String,
        json: String,
    },
    /// Replace schedule parameters with any JSON value
    Parameters {
        id: String,
        json: String,
    },
    Pause {
        id: String,
    },
    Resume {
        id: String,
    },
    Cancel {
        id: String,
    },
    /// Start an immediate run
    Run {
        id: String,
    },
    /// List runs for a schedule
    Runs {
        id: String,
    },
    /// Inspect an occurrence, including its output
    RunShow {
        id: String,
    },
    CancelRun {
        id: String,
    },
    /// Retry a failed or interrupted occurrence; --confirm is required for interrupted runs
    Retry {
        id: String,
        #[arg(long)]
        confirm: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Args::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(args: Args) -> Result<()> {
    let (method, path, body) = match args.command {
        Command::Create {
            program,
            args: argv,
            at,
            cron,
            name,
            parameters,
            missed_policy,
            max_retries,
            retry_delays,
            trigger_on_parameters,
            trigger_behavior,
        } => {
            let timing = match (at, cron) {
                (Some(at), None) => json!({"type":"once", "at": at}),
                (None, Some(expression)) => json!({"type":"cron", "expression": expression}),
                (None, None) => bail!("provide --at RFC3339 or --cron EXPR"),
                _ => unreachable!(),
            };
            let parameters: Value =
                serde_json::from_str(&parameters).context("invalid --parameters JSON")?;
            let mut body = json!({"program":program,"args":argv,"timing":timing,"parameters":parameters,"missedPolicy":missed_policy,"retryPolicy":{"maxRetries":max_retries,"delaysSeconds":retry_delays},"parameterUpdateTrigger":{"enabled":trigger_on_parameters,"scheduleBehavior":trigger_behavior}});
            if let Some(name) = name {
                body["name"] = json!(name);
            }
            ("POST", "/schedules".to_owned(), Some(body))
        }
        Command::Schedules => ("GET", "/schedules?limit=100".to_owned(), None),
        Command::Show { id } => ("GET", format!("/schedules/{id}"), None),
        Command::Update { id, json } => (
            "PATCH",
            format!("/schedules/{id}"),
            Some(serde_json::from_str(&json).context("invalid JSON object")?),
        ),
        Command::Parameters { id, json } => (
            "PUT",
            format!("/schedules/{id}/parameters"),
            Some(serde_json::from_str(&json).context("invalid JSON value")?),
        ),
        Command::Pause { id } => ("POST", format!("/schedules/{id}/pause"), None),
        Command::Resume { id } => ("POST", format!("/schedules/{id}/resume"), None),
        Command::Cancel { id } => ("POST", format!("/schedules/{id}/cancel"), None),
        Command::Run { id } => ("POST", format!("/schedules/{id}/runs"), None),
        Command::Runs { id } => (
            "GET",
            format!("/schedules/{id}/occurrences?limit=100"),
            None,
        ),
        Command::RunShow { id } => ("GET", format!("/occurrences/{id}"), None),
        Command::CancelRun { id } => ("POST", format!("/occurrences/{id}/cancel"), None),
        Command::Retry { id, confirm } => (
            "POST",
            format!("/occurrences/{id}/retry"),
            Some(json!({"confirmInterrupted":confirm})),
        ),
    };
    let base = args.server.trim_end_matches('/');
    validate_server_url(base)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("could not configure HTTP client")?;
    let mut value = request_json(&client, base, method, &path, body.as_ref()).await?;
    if method == "GET"
        && (path == "/schedules?limit=100" || path.ends_with("/occurrences?limit=100"))
    {
        let mut items = value["items"].as_array().cloned().unwrap_or_default();
        let mut cursor = value["nextCursor"].as_str().map(str::to_owned);
        while let Some(next) = cursor {
            let separator = if path.contains('?') { '&' } else { '?' };
            let next_path = format!("{path}{separator}cursor={next}");
            let page = request_json(&client, base, "GET", &next_path, None).await?;
            items.extend(page["items"].as_array().cloned().unwrap_or_default());
            cursor = page["nextCursor"].as_str().map(str::to_owned);
        }
        value["items"] = Value::Array(items);
        value["nextCursor"] = Value::Null;
    }
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, &value)?;
    writeln!(&mut stdout)?;
    Ok(())
}

fn validate_server_url(value: &str) -> Result<reqwest::Url> {
    let server = reqwest::Url::parse(value).context("--server must be a valid URL")?;
    let host = server.host_str().unwrap_or_default();
    let loopback = host == "localhost"
        || host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback());
    if !loopback || server.scheme() != "http" {
        bail!("--server must use HTTP on localhost or a loopback IP");
    }
    Ok(server)
}

async fn request_json(
    client: &reqwest::Client,
    base: &str,
    method: &str,
    path: &str,
    body: Option<&Value>,
) -> Result<Value> {
    let url = format!("{base}/api/v1{path}");
    let mut request = client.request(method.parse()?, url);
    if let Some(body) = body {
        request = request.json(body);
    }
    let response = request
        .send()
        .await
        .context("could not reach the local daemon")?;
    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .context("could not read daemon response")?;
    if !status.is_success() {
        let detail = serde_json::from_slice::<Value>(&bytes)
            .ok()
            .and_then(|v| v.get("detail").and_then(Value::as_str).map(str::to_owned))
            .unwrap_or_else(|| String::from_utf8_lossy(&bytes).into_owned());
        bail!("daemon returned {status}: {detail}");
    }
    if bytes.is_empty() {
        Ok(Value::Null)
    } else {
        serde_json::from_slice(&bytes).context("daemon returned invalid JSON")
    }
}

#[cfg(test)]
mod tests {
    use super::validate_server_url;

    #[test]
    fn accepts_local_daemon_urls() {
        for url in [
            "http://127.0.0.1:30002",
            "http://[::1]:30002",
            "http://localhost:30002",
        ] {
            assert!(validate_server_url(url).is_ok(), "{url}");
        }
    }

    #[test]
    fn rejects_non_loopback_daemon_urls() {
        assert!(validate_server_url("http://192.0.2.10:30002").is_err());
        assert!(validate_server_url("ftp://localhost:30002").is_err());
    }
}
