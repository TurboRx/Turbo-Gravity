use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Form, Router,
};
use serde::{Deserialize, Serialize};

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
    let data = SetupData {
        owner_id: state.config.dashboard.admin_ids.first().cloned().unwrap_or_default(),
        client_id: state.config.bot.client_id.clone(),
        client_secret: state.config.dashboard.client_secret.clone(),
        callback_url: state.config.dashboard.callback_url.clone(),
        mongo_uri: state.config.database.mongo_uri.clone(),
        session_secret: state.config.dashboard.session_secret.clone(),
        guild_id: state.config.bot.guild_id.clone(),
        port: state.config.dashboard.port,
        presence_type: state.config.bot.presence_type,
        presence_text: state.config.bot.presence_text.clone(),
        command_scope: state.config.bot.command_scope.clone(),
    };
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
// Setup form submission
// ---------------------------------------------------------------------------

/// Form fields submitted from the /setup page.
#[derive(Deserialize)]
pub struct SetupForm {
    #[serde(rename = "botToken", default)]
    pub bot_token: String,
    #[serde(rename = "clientId", default)]
    pub client_id: String,
    #[serde(rename = "clientSecret", default)]
    pub client_secret: String,
    #[serde(rename = "callbackUrl", default)]
    pub callback_url: String,
    #[serde(rename = "mongoUri", default)]
    pub mongo_uri: String,
    #[serde(rename = "sessionSecret", default)]
    pub session_secret: String,
    #[serde(rename = "adminIds", default)]
    pub admin_ids: String,
    #[serde(rename = "guildId", default)]
    pub guild_id: String,
    #[serde(default = "default_port_str")]
    pub port: String,
    #[serde(rename = "presenceType", default)]
    pub presence_type: String,
    #[serde(rename = "presenceText", default)]
    pub presence_text: String,
    #[serde(rename = "commandScope", default)]
    pub command_scope: String,
}

fn default_port_str() -> String {
    "8080".to_string()
}

/// `POST /setup` — save the wizard form to `config.toml` and redirect to `/dashboard`.
async fn setup_submit(Form(form): Form<SetupForm>) -> Response {
    use crate::config::{BotConfig, Config, DashboardConfig, DatabaseConfig};

    let port: u16 = form.port.parse().unwrap_or(8080);
    let presence_type: u8 = form.presence_type.parse().unwrap_or(0);

    let admin_ids: Vec<String> = form
        .admin_ids
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    let callback_url = if form.callback_url.is_empty() {
        format!("http://localhost:{port}/auth/discord/callback")
    } else {
        form.callback_url.clone()
    };

    let command_scope = if form.command_scope.is_empty() {
        crate::config::DEFAULT_COMMAND_SCOPE.to_string()
    } else {
        form.command_scope.clone()
    };

    let presence_text = if form.presence_text.is_empty() {
        crate::config::DEFAULT_PRESENCE_TEXT.to_string()
    } else {
        form.presence_text.clone()
    };

    let cfg = Config {
        bot: BotConfig {
            token: form.bot_token.clone(),
            client_id: form.client_id.clone(),
            guild_id: form.guild_id.clone(),
            command_scope,
            presence_text,
            presence_type,
        },
        database: DatabaseConfig {
            mongo_uri: form.mongo_uri.clone(),
        },
        dashboard: DashboardConfig {
            enable_dashboard: true,
            port,
            session_secret: form.session_secret.clone(),
            client_secret: form.client_secret.clone(),
            callback_url,
            admin_ids,
        },
    };

    match crate::config::save(&cfg) {
        Ok(()) => (
            StatusCode::FOUND,
            [(header::LOCATION, "/dashboard")],
        )
            .into_response(),
        Err(e) => {
            let data = ErrorData {
                code: 500,
                title: "Setup Failed".to_string(),
                message: format!("Could not save config.toml: {e}"),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Html(pages::error_page(&data))).into_response()
        }
    }
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
        .route("/setup", get(setup_page).post(setup_submit))
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

    const TEST_MAX_BODY_SIZE: usize = 4 * 1024 * 1024;

    async fn body_string(b: Body) -> String {
        let bytes = body::to_bytes(b, TEST_MAX_BODY_SIZE).await.unwrap();
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

    // ------------------------------------------------------------------
    // POST /setup
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn setup_post_with_valid_form_saves_config_and_redirects() {
        use axum::http::{header, Method};

        // Write a temporary config.toml so the save path exists during the test
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "").unwrap();

        // Change working directory to temp_dir so config::save writes there
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let body = "botToken=mytoken&clientId=appid&clientSecret=&callbackUrl=http%3A%2F%2Flocalhost%3A8080%2Fauth%2Fdiscord%2Fcallback&mongoUri=&sessionSecret=&adminIds=&guildId=&port=8080&presenceType=0&presenceText=Ready&commandScope=guild";

        let resp = test_app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/setup")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Restore working directory
        std::env::set_current_dir(&original_dir).unwrap();

        // On success the handler redirects to /dashboard
        assert_eq!(resp.status(), StatusCode::FOUND);
        assert_eq!(resp.headers().get("location").unwrap(), "/dashboard");

        // Verify config.toml was written with the submitted values
        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(written.contains("mytoken"));
        assert!(written.contains("appid"));
    }

    #[tokio::test]
    async fn setup_page_get_prepopulates_existing_config() {
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
        let text = body_string(resp.into_body()).await;
        // The test state has client_id "123456" which should appear in the pre-filled form
        assert!(text.contains("123456"));
    }
}
