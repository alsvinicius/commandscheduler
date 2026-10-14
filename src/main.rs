use std::{net::SocketAddr, path::PathBuf};

use anyhow::Context;
use axum::{Router, http::header, response::IntoResponse, routing::get};
use tower_http::services::{ServeDir, ServeFile};

pub mod api;
mod database;
pub mod execution;
mod process_tree;
pub mod recurrence;
pub mod scheduler;
pub mod storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let address: SocketAddr = std::env::var("COMMAND_SCHEDULER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:4371".to_owned())
        .parse()
        .context("COMMAND_SCHEDULER_ADDR must be a socket address")?;
    anyhow::ensure!(
        address.ip().is_loopback(),
        "COMMAND_SCHEDULER_ADDR must use a loopback address"
    );

    let _daemon_lock = storage::acquire_daemon_lock()?;
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .with_context(|| format!("could not bind local server at {address}"))?;
    let _database = storage::connect_default().await?;
    storage::start_retention_worker(_database.clone()).await?;
    let interrupted = scheduler::recover_interrupted(&_database).await?;
    if interrupted > 0 {
        eprintln!("{interrupted} occurrence(s) require manual retry after daemon restart");
    }
    scheduler::start(_database.clone());

    let assets = std::env::var_os("COMMAND_SCHEDULER_WEB_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|parent| parent.join("ui")))
                .filter(|path| path.join("index.html").is_file())
        })
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/dist"));
    let index = assets.join("index.html");

    let api_state = api::ApiState {
        pool: _database.clone(),
        cancellation: std::sync::Arc::new(execution::request_cancel),
    };
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/openapi.yaml", get(openapi_spec))
        .nest("/api/v1", api::router(api_state))
        .route_service("/docs/api", ServeFile::new(index.clone()))
        .route_service("/docs/api/", ServeFile::new(index.clone()))
        .fallback_service(ServeDir::new(assets).not_found_service(ServeFile::new(index)));

    println!("Command Scheduler listening at http://{address}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server stopped unexpectedly")?;
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut terminate = signal(SignalKind::terminate()).expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

async fn openapi_spec() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/yaml; charset=utf-8")],
        include_str!("../docs/engineering/api/openapi.yaml"),
    )
}
