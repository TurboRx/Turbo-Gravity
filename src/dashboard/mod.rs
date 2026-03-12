pub mod pages;
pub mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::state::SharedState;

/// Spin up the optional Axum dashboard API.
///
/// In **setup mode** (called from `main` before a bot token is configured) the
/// server binds to `127.0.0.1` (loopback only) because the setup page and the
/// backup/restore endpoints handle unencrypted bot tokens and config secrets.
///
/// In **normal mode** the server binds to `0.0.0.0` so it is reachable from
/// the Docker host or a reverse-proxy.  The CORS layer restricts browser
/// cross-origin access to localhost, but non-browser clients can still reach
/// every endpoint.  If you expose the dashboard externally, place it behind an
/// authenticating reverse proxy.
pub async fn serve(state: SharedState) -> anyhow::Result<()> {
    let port = state.config.dashboard.port;

    // In setup mode the config is not yet complete, so no bot token is present.
    // Bind to loopback only to prevent the unprotected setup wizard and the
    // backup/restore endpoints from being reachable from non-local addresses.
    let is_setup_mode = crate::config::needs_setup(&state.config);
    let bind_addr = if is_setup_mode {
        format!("127.0.0.1:{port}")
    } else {
        tracing::warn!(
            "Dashboard is listening on all interfaces (0.0.0.0:{port}). \
             The config backup/restore endpoints expose bot secrets. \
             Place the dashboard behind an authenticating reverse proxy if it is publicly reachable."
        );
        format!("0.0.0.0:{port}")
    };

    let app = Router::new()
        .merge(routes::router())
        // Restrict CORS to localhost only for security
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
    // into each request's extensions so that the `require_loopback` middleware
    // on the backup/restore routes can enforce localhost-only access.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await?;
    Ok(())
}
