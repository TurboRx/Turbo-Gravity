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

    // Load config.toml (or a default unconfigured config if the file is absent).
    // Validation is deferred until we know the bot is actually configured.
    let cfg = config::load()?;

    // -- Setup mode -----------------------------------------------------------
    // If the bot token has not been set yet (fresh clone / first run), skip
    // Discord entirely and start the setup wizard so the user can configure
    // everything through a browser before the bot tries to connect.
    if config::needs_setup(&cfg) {
        info!("No bot token configured -- entering setup mode");
        info!(
            "Open http://127.0.0.1:{}/setup in your browser to configure the bot.",
            cfg.dashboard.port
        );
        info!("Once you save the configuration, restart the bot to connect to Discord.");

        let state = Arc::new(state::AppState::new(cfg.clone(), None));
        let dashboard_state = Arc::clone(&state);
        info!(
            "Setup dashboard listening on http://0.0.0.0:{}",
            cfg.dashboard.port
        );

        // In setup mode, the dashboard is the only active component.
        // Run it in the foreground so that startup failures propagate
        // instead of leaving the process hanging while waiting for a
        // shutdown signal.
        dashboard::serve(dashboard_state).await?;
        info!("Shutting down setup wizard. Run the bot again after saving your configuration.");
        return Ok(());
    }
    }
    // -------------------------------------------------------------------------

    // Full validation -- only reached when the bot token is present.
    config::validate(&cfg)?;
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

    // Start the Poise Discord bot (blocks until shutdown or signal)
    tokio::select! {
        res = bot::start(Arc::clone(&state)) => {
            if let Err(e) = res {
                tracing::error!("Bot exited with error: {e}");
            }
        }
        _ = shutdown_signal() => {
            info!("Shutdown signal received, exiting gracefully");
        }
    }

    Ok(())
}

/// Wait for SIGINT (Ctrl-C) or SIGTERM so the process can shut down cleanly.
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
