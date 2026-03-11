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
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        // Inject Arc<AppState> into all route handlers
        .with_state(Arc::clone(&state));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    info!("Dashboard listening on http://0.0.0.0:{port}");

    axum::serve(listener, app).await?;
    Ok(())
}
