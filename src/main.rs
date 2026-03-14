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
    let mut cfg = config::load()?;

    // Allow the PORT environment variable to override the dashboard port.
    // Cloud platforms can set PORT to the port they route
    // external traffic to; the app must listen on that port to be reachable.
    if let Ok(port_str) = std::env::var("PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            // Port 0 is excluded: it asks the OS to assign a random port,
            // which is not useful for a predictable dashboard endpoint.
            if port > 0 {
                cfg.dashboard.port = port;
            }
        }
    }

    // -- Setup mode -----------------------------------------------------------
    // If the bot is not yet fully configured (fresh clone / first run), skip
    // Discord entirely and start the setup wizard so the user can configure
    // everything through a browser before the bot tries to connect.
    if config::needs_setup(&cfg) {
        // Fall back to the default port if the configured port is invalid (e.g. 0).
        let setup_port = if cfg.dashboard.port > 0 {
            cfg.dashboard.port
        } else {
            config::DEFAULT_PORT
        };

        info!("No bot token configured -- entering setup mode");
        info!(
            "Setup wizard listening on port {}. Open /setup on your server's public URL to configure the bot.",
            setup_port
        );

        // Build a state with the (possibly corrected) port so the server binds correctly.
        let mut setup_cfg = cfg.clone();
        setup_cfg.dashboard.port = setup_port;
        let state = Arc::new(state::AppState::new(setup_cfg, None));

        // In setup mode, the dashboard is the only active component.
        // It will shut down automatically once the user saves the configuration
        // (signalled via `AppState::setup_complete`).
        dashboard::serve(state).await?;

        // Re-load the config that was just saved by the wizard.
        let new_cfg = config::load()?;
        if config::needs_setup(&new_cfg) {
            // The wizard was dismissed without completing setup (e.g. server
            // process was interrupted before the form was submitted).
            info!("Setup not completed. Exiting. Run the bot again to re-enter setup mode.");
            return Ok(());
        }

        info!("Setup complete — starting the bot automatically…");
        cfg = new_cfg;
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
