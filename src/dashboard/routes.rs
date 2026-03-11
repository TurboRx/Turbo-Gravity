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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::{config, state};

    fn test_state() -> state::SharedState {
        let cfg: config::Config = toml::from_str(
            r#"
[bot]
token = "test-token"
client_id = "123456"
guild_id = "999"

[dashboard]
enable_dashboard = true
port = 8080
"#,
        )
        .expect("test config must parse");
        Arc::new(state::AppState::new(cfg, None))
    }

    fn test_app() -> axum::Router {
        router().with_state(test_state())
    }

    async fn body_string(b: Body) -> String {
        let bytes = body::to_bytes(b, usize::MAX).await.unwrap();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    // ------------------------------------------------------------------
    // /health
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn health_returns_200() {
        let resp = test_app()
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_body_contains_ok_status() {
        let resp = test_app()
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let text = body_string(resp.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["status"], "ok");
        assert!(v["version"].is_string());
    }

    // ------------------------------------------------------------------
    // /api/stats
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn api_stats_returns_200() {
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .uri("/api/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_stats_bot_configured_true_when_token_set() {
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .uri("/api/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let text = body_string(resp.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["bot_configured"], true);
        assert_eq!(v["database_connected"], false);
        assert_eq!(v["dashboard_port"], 8080);
    }

    #[tokio::test]
    async fn api_stats_bot_configured_false_when_no_token() {
        let cfg: config::Config = toml::from_str(
            r#"
[bot]
token = ""
client_id = "123"
"#,
        )
        .unwrap();
        let state = Arc::new(state::AppState::new(cfg, None));
        let app = router().with_state(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let text = body_string(resp.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["bot_configured"], false);
    }

    // ------------------------------------------------------------------
    // /api/config
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn api_config_returns_200() {
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .uri("/api/config")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_config_fields() {
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .uri("/api/config")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let text = body_string(resp.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["command_scope"], "guild");
        assert_eq!(v["guild_id"], "999");
        assert_eq!(v["dashboard_port"], 8080);
        assert_eq!(v["enable_dashboard"], true);
    }

    // ------------------------------------------------------------------
    // HTML pages
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn root_redirects_to_dashboard() {
        let resp = test_app()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FOUND);
        assert_eq!(
            resp.headers().get("location").unwrap(),
            "/dashboard"
        );
    }

    #[tokio::test]
    async fn dashboard_page_returns_html() {
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .uri("/dashboard")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("text/html"), "expected text/html, got {ct}");
    }

    #[tokio::test]
    async fn setup_page_returns_html() {
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .uri("/setup")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn selector_page_returns_html() {
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .uri("/selector")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ------------------------------------------------------------------
    // Static asset
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn styles_css_returns_css_content_type() {
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .uri("/styles.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("text/css"), "expected text/css, got {ct}");
    }

    // ------------------------------------------------------------------
    // Fallback / 404
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .uri("/this-route-does-not-exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
