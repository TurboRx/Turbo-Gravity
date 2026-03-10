mod bot;
mod config;
mod dashboard;
mod db;
mod state;

use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise structured logging from RUST_LOG env var (defaults to "info")
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load optional .env file (ignored if absent)
    let _ = dotenvy::dotenv();

    // Load config.toml
    let cfg = config::load()?;
    info!("Configuration loaded");

    // Connect to MongoDB (optional – bot runs without a DB if uri is empty)
    let db_client = if cfg.database.mongo_uri.is_empty() {
        info!("No MongoDB URI configured – running without database");
        None
    } else {
        let client = db::connect(&cfg.database.mongo_uri).await?;
        info!("Connected to MongoDB");
        Some(client)
    };

    // Build shared application state wrapped in Arc for cheap cloning
    let state = Arc::new(state::AppState::new(cfg.clone(), db_client));

    // Conditionally start the Axum dashboard
    if cfg.dashboard.enable_dashboard {
        let dashboard_state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = dashboard::serve(dashboard_state).await {
                tracing::error!("Dashboard error: {e}");
            }
        });
        info!(
            "Dashboard enabled – listening on port {}",
            cfg.dashboard.port
        );
    } else {
        info!("Dashboard disabled (enable_dashboard = false in config.toml)");
    }

    // Start the Poise Discord bot (blocks until shutdown)
    bot::start(Arc::clone(&state)).await?;

    Ok(())
}
