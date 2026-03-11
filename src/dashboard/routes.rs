use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;

use crate::state::SharedState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub bot_configured: bool,
    pub database_connected: bool,
    pub dashboard_port: u16,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /health — simple liveness check
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// GET /api/stats — basic bot/dashboard stats
///
/// Demonstrates the `State(state): State<Arc<AppState>>` Axum extractor
/// pattern for sharing data between routes — modelled on the axum chat example.
async fn stats(State(state): State<SharedState>) -> Json<StatsResponse> {
    Json(StatsResponse {
        bot_configured: !state.config.bot.token.is_empty(),
        database_connected: state.db.is_some(),
        dashboard_port: state.config.dashboard.port,
    })
}

/// GET /api/config — returns public (non-secret) configuration fields
#[derive(Serialize)]
struct PublicConfig {
    command_scope: String,
    presence_text: String,
    guild_id: String,
    dashboard_port: u16,
    enable_dashboard: bool,
}

async fn public_config(State(state): State<SharedState>) -> Json<PublicConfig> {
    Json(PublicConfig {
        command_scope: state.config.bot.command_scope.clone(),
        presence_text: state.config.bot.presence_text.clone(),
        guild_id: state.config.bot.guild_id.clone(),
        dashboard_port: state.config.dashboard.port,
        enable_dashboard: state.config.dashboard.enable_dashboard,
    })
}

/// Fallback handler for unknown routes
async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Build the Axum router.  State is attached in `dashboard::serve()` so this
/// function only declares routes — keeping concerns separated.
pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/health", get(health))
        .route("/api/stats", get(stats))
        .route("/api/config", get(public_config))
        .fallback(not_found)
}
