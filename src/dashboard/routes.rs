use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde::Serialize;

use crate::state::SharedState;
use super::pages::{
    self, DashboardData, ErrorData, SelectorData, SetupData,
};

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
// JSON API handlers
// ---------------------------------------------------------------------------

/// GET /health — simple liveness check
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// GET /api/stats — basic bot/dashboard stats
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

// ---------------------------------------------------------------------------
// HTML page handlers
// ---------------------------------------------------------------------------

/// GET /styles.css — embedded stylesheet (replaces the former static CSS file)
async fn styles() -> Response {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        pages::STYLES,
    )
        .into_response()
}

/// GET / — redirect to the dashboard page
async fn root() -> Response {
    (
        StatusCode::FOUND,
        [(header::LOCATION, "/dashboard")],
    )
        .into_response()
}

/// GET /dashboard — main control-panel page
async fn dashboard_page(State(state): State<SharedState>) -> Html<String> {
    let bot_status = if !state.config.bot.token.is_empty() {
        "online"
    } else {
        "offline"
    };
    let permissions = "8".to_string();
    let invite_link = if !state.config.bot.client_id.is_empty() {
        format!(
            "https://discord.com/api/oauth2/authorize?client_id={}&permissions={}&scope=bot%20applications.commands",
            state.config.bot.client_id, permissions,
        )
    } else {
        "#".to_string()
    };
    let data = DashboardData {
        bot_status,
        command_scope: state.config.bot.command_scope.clone(),
        guild_id: state.config.bot.guild_id.clone(),
        invite_link,
        invite_permissions: permissions,
    };
    Html(pages::dashboard_page(&data))
}

/// GET /setup — first-run setup wizard page
async fn setup_page(State(state): State<SharedState>) -> Html<String> {
    let owner_id = state.config.dashboard.admin_ids.first().cloned().unwrap_or_default();
    let data = SetupData { owner_id };
    Html(pages::setup_page(&data))
}

/// GET /selector — guild selector page
async fn selector_page(State(state): State<SharedState>) -> Html<String> {
    let bot_status = if !state.config.bot.token.is_empty() {
        "online"
    } else {
        "offline"
    };
    let data = SelectorData {
        username: "Admin".to_string(),
        guilds: vec![],
        bot_status,
    };
    Html(pages::selector_page(&data))
}

/// Fallback handler — renders the HTML error page
async fn not_found() -> Response {
    let data = ErrorData {
        code: 404,
        title: "Not Found".to_string(),
        message: "The page you're looking for doesn't exist.".to_string(),
    };
    (StatusCode::NOT_FOUND, Html(pages::error_page(&data))).into_response()
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Build the Axum router.  State is attached in `dashboard::serve()` so this
/// function only declares routes — keeping concerns separated.
pub fn router() -> Router<SharedState> {
    Router::new()
        // Static assets
        .route("/styles.css", get(styles))
        // HTML pages
        .route("/", get(root))
        .route("/dashboard", get(dashboard_page))
        .route("/setup", get(setup_page))
        .route("/selector", get(selector_page))
        // JSON API
        .route("/health", get(health))
        .route("/api/stats", get(stats))
        .route("/api/config", get(public_config))
        .fallback(not_found)
}
