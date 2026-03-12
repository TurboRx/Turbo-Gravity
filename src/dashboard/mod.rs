pub mod pages;
pub mod routes;

use std::sync::Arc;

use axum::{Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::state::SharedState;

/// Spin up the optional Axum dashboard API.
///
/// Bound to `config.dashboard.port`; the `Arc<AppState>` is injected into
/// every route handler via Axum's `State` extractor — the exact shared-state
/// pattern used in the tokio-rs/axum chat example.
pub async fn serve(state: SharedState) -> anyhow::Result<()> {
    let port = state.config.dashboard.port;

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

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    info!("Dashboard listening on http://0.0.0.0:{port}");

    // When the setup wizard successfully saves a configuration it calls
    // `state.setup_complete.notify_one()`.  We use Axum's graceful-shutdown
    // hook to stop the server at that point so that `main` can re-read the
    // freshly written config and start the bot automatically.
    let shutdown = {
        let state = Arc::clone(&state);
        async move { state.setup_complete.notified().await }
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}
