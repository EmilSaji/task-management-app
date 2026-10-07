use anyhow::Context;
use task_api::{build_app, config::Config, AppState};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("task_api=debug,tower_http=info")),
        )
        .init();

    let config = Config::from_env()?;
    let bind_addr = config.bind_addr.clone();
    let dev = config.is_development();

    let state = AppState::connect(config).await?;
    let app = build_app(state);

    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind {bind_addr}"))?;
    tracing::info!("listening on http://{bind_addr}");
    tracing::info!("swagger UI at http://{bind_addr}/swagger-ui");
    if dev {
        tracing::info!("development mode: /seed/users and /dev/email-logs/latest are enabled");
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}
