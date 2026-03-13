pub mod auth;
pub mod pages;
pub mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::state::SharedState;

/// Spin up the optional Axum dashboard API.
///
/// The server always binds to `0.0.0.0:{port}` (required for Zeabur and other
/// container environments).  The `/api/config/*` routes and `/dashboard/settings`
/// are protected by the `require_admin` middleware which enforces a valid Discord
/// OAuth2 session belonging to `ADMIN_DISCORD_ID`.
pub async fn serve(state: SharedState) -> anyhow::Result<()> {
    let port = state.config.dashboard.port;

    // Always bind to all interfaces so the dashboard is reachable inside
    // containers (Zeabur, Docker) and behind reverse proxies.
    let bind_addr = format!("0.0.0.0:{port}");

    // Build protected sub-routers.
    // The require_admin middleware validates the session cookie and ensures only
    // the configured ADMIN_DISCORD_ID can reach these routes.
    let admin_guard = axum::middleware::from_fn_with_state(Arc::clone(&state), auth::require_admin);

    let config_routes = routes::config_router()
        .route_layer(admin_guard.clone());
    let admin_routes = routes::admin_router()
        .route_layer(admin_guard);

    let app = Router::new()
        // Public routes (setup wizard, dashboard pages, health, control, etc.)
        .merge(routes::public_router())
        // Protected /api/config/* routes (backup, restore, config view)
        .nest("/api/config", config_routes)
        // Protected admin routes (/dashboard/settings, etc.)
        .merge(admin_routes)
        // Discord OAuth2 flow
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        // Allow CORS requests from the same host on both HTTP and HTTPS.
        // This covers local development (localhost) and cloud deployments where
        // the browser and API share the same origin (same-origin requests bypass
        // CORS entirely, so this layer mainly guards against cross-origin callers).
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:8080".parse().unwrap(),
                    format!("http://localhost:{port}").parse().unwrap(),
                    "http://127.0.0.1:8080".parse().unwrap(),
                    format!("http://127.0.0.1:{port}").parse().unwrap(),
                ])
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                .allow_headers([axum::http::header::CONTENT_TYPE]),
        )
        .layer(TraceLayer::new_for_http())
        // Inject Arc<AppState> into all route handlers
        .with_state(Arc::clone(&state));

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Dashboard listening on http://{bind_addr}");

    // When the setup wizard successfully saves a configuration it calls
    // `state.setup_complete.notify_one()`.  We use Axum's graceful-shutdown
    // hook to stop the server at that point so that `main` can re-read the
    // freshly written config and start the bot automatically.
    let shutdown = {
        let state = Arc::clone(&state);
        async move { state.setup_complete.notified().await }
    };

    // `into_make_service_with_connect_info` injects `ConnectInfo<SocketAddr>`
    // into each request's extensions (used by fallback handlers and tracing).
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await?;
    Ok(())
}
