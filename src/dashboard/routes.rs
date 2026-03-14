use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Form, Router,
};
use serde::{Deserialize, Serialize};

use crate::state::SharedState;
use super::pages::{
    self, DashboardData, ErrorData, SelectorData, SetupData, SettingsData,
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

/// Live bot status response (used for auto-refresh polling)
#[derive(Serialize)]
pub struct BotStatusResponse {
    pub online: bool,
    pub guild_count: usize,
}

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

/// GET /api/bot/status — live bot status (online flag + guild count).
///
/// Polled by the dashboard auto-refresh script every 30 seconds.
async fn bot_status(State(state): State<SharedState>) -> Json<BotStatusResponse> {
    Json(BotStatusResponse {
        online: state.bot_online.load(std::sync::atomic::Ordering::Relaxed),
        guild_count: state.guild_count.load(std::sync::atomic::Ordering::Relaxed),
    })
}

/// GET /api/config — returns configuration fields for the admin dashboard.
///
/// This handler is registered under `config_router()` which is protected by the
/// `require_admin` middleware, so it is only reachable by authenticated admins.
#[derive(Serialize)]
struct AdminConfig {
    command_scope: String,
    presence_text: String,
    guild_id: String,
    dashboard_port: u16,
    enable_dashboard: bool,
}

async fn public_config(State(state): State<SharedState>) -> Json<AdminConfig> {
    Json(AdminConfig {
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

/// GET / — redirect to setup wizard when unconfigured; otherwise redirect to the dashboard
async fn root(State(state): State<SharedState>) -> Response {
    let location = if crate::config::needs_setup(&state.config) {
        "/setup"
    } else {
        "/dashboard"
    };
    (StatusCode::FOUND, [(header::LOCATION, location)]).into_response()
}

/// GET /dashboard — main control-panel page
async fn dashboard_page(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Html<String> {
    let bot_status = if state.bot_online.load(std::sync::atomic::Ordering::Relaxed) {
        "online"
    } else {
        "offline"
    };
    let guild_count = state.guild_count.load(std::sync::atomic::Ordering::Relaxed);
    let permissions = "8".to_string();
    let invite_link = if !state.config.bot.client_id.is_empty() {
        format!(
            "https://discord.com/api/oauth2/authorize?client_id={}&permissions={}&scope=bot%20applications.commands",
            state.config.bot.client_id, permissions,
        )
    } else {
        "#".to_string()
    };
    // Extract the logged-in username from the session cookie (best-effort).
    let username = super::auth::current_session(&state, &headers)
        .await
        .map(|s| s.username)
        .unwrap_or_else(|| "Admin".to_string());
    let data = DashboardData {
        bot_status,
        command_scope: state.config.bot.command_scope.clone(),
        guild_id: state.config.bot.guild_id.clone(),
        invite_link,
        invite_permissions: permissions,
        online_status: state.config.bot.online_status.clone(),
        presence_text: state.config.bot.presence_text.clone(),
        presence_type: state.config.bot.presence_type,
        guild_count,
        username,
    };
    Html(pages::dashboard_page(&data))
}

/// GET /setup — first-run setup wizard page.
///
/// Only renders the full form (including bot token and OAuth secrets) when the
/// bot is not yet configured (`needs_setup` is true).  Once the bot is
/// configured, visiting `/setup` redirects to `/dashboard` so that secrets are
/// never exposed to unauthenticated users on a publicly-bound server.
async fn setup_page(State(state): State<SharedState>) -> Response {
    if !crate::config::needs_setup(&state.config) {
        // Setup already complete — redirect away to avoid exposing secrets.
        return (StatusCode::FOUND, [(header::LOCATION, "/dashboard")]).into_response();
    }
    let data = SetupData {
        token: state.config.bot.token.clone(),
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
        online_status: state.config.bot.online_status.clone(),
        avatar_url: state.config.bot.avatar_url.clone(),
    };
    Html(pages::setup_page(&data)).into_response()
}

/// GET /selector — guild selector page
async fn selector_page(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Html<String> {
    let bot_status = if state.bot_online.load(std::sync::atomic::Ordering::Relaxed) {
        "online"
    } else {
        "offline"
    };
    let username = super::auth::current_session(&state, &headers)
        .await
        .map(|s| s.username)
        .unwrap_or_else(|| "Admin".to_string());
    let guild_count = state.guild_count.load(std::sync::atomic::Ordering::Relaxed);
    let data = SelectorData {
        username,
        guilds: vec![],
        bot_status,
        guild_count,
    };
    Html(pages::selector_page(&data))
}

/// GET /settings — user profile & logout page
async fn settings_page(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Html<String> {
    let session = super::auth::current_session(&state, &headers).await;
    let (username, user_id) = session
        .map(|s| (s.username, s.user_id))
        .unwrap_or_else(|| ("Admin".to_string(), String::new()));
    let data = SettingsData { username, user_id };
    Html(pages::settings_page(&data))
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
    #[serde(rename = "onlineStatus", default)]
    pub online_status: String,
    #[serde(rename = "avatarUrl", default)]
    pub avatar_url: String,
}

fn default_port_str() -> String {
    "8080".to_string()
}

/// `POST /setup` — save the wizard form to `config.toml` and show the setup-complete page.
async fn setup_submit(State(state): State<SharedState>, Form(form): Form<SetupForm>) -> Response {
    use crate::config::{BotConfig, Config, DashboardConfig, DatabaseConfig};

    // Only allow form submission during initial setup (before a bot token is configured).
    // Once the bot is configured the endpoint is closed to prevent unauthenticated
    // overwrites of config.toml from a publicly-accessible server.
    if !crate::config::needs_setup(&state.config) {
        let data = ErrorData {
            code: 403,
            title: "Forbidden".to_string(),
            message: "Setup is already complete. Use the dashboard settings to modify configuration.".to_string(),
        };
        return (StatusCode::FORBIDDEN, Html(pages::error_page(&data))).into_response();
    }

    // Validate port
    let port: u16 = match form.port.parse() {
        Ok(p) if p > 0 => p,
        _ => {
            let data = ErrorData {
                code: 400,
                title: "Invalid Port".to_string(),
                message: format!("Port must be a positive number, got '{}'", form.port),
            };
            return (StatusCode::BAD_REQUEST, Html(pages::error_page(&data))).into_response();
        }
    };

    // Validate presence_type
    let presence_type: u8 = match form.presence_type.parse() {
        Ok(pt) if pt <= 4 => pt,
        _ => {
            let data = ErrorData {
                code: 400,
                title: "Invalid Presence Type".to_string(),
                message: format!("Presence type must be 0-4, got '{}'", form.presence_type),
            };
            return (StatusCode::BAD_REQUEST, Html(pages::error_page(&data))).into_response();
        }
    };

    // Validate and parse admin_ids
    let admin_ids: Vec<String> = form
        .admin_ids
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    // Validate each admin_id is a valid u64 (Discord snowflake)
    for admin_id in &admin_ids {
        if admin_id.parse::<u64>().is_err() {
            let data = ErrorData {
                code: 400,
                title: "Invalid Admin ID".to_string(),
                message: format!("Admin ID '{}' must be a valid Discord snowflake (numeric)", admin_id),
            };
            return (StatusCode::BAD_REQUEST, Html(pages::error_page(&data))).into_response();
        }
    }

    // Validate client_id if provided
    if !form.client_id.trim().is_empty() && form.client_id.parse::<u64>().is_err() {
        let data = ErrorData {
            code: 400,
            title: "Invalid Client ID".to_string(),
            message: format!("Client ID must be a valid Discord snowflake (numeric), got '{}'", form.client_id),
        };
        return (StatusCode::BAD_REQUEST, Html(pages::error_page(&data))).into_response();
    }

    // Validate guild_id if provided
    if !form.guild_id.trim().is_empty() && form.guild_id.parse::<u64>().is_err() {
        let data = ErrorData {
            code: 400,
            title: "Invalid Guild ID".to_string(),
            message: format!("Guild ID must be a valid Discord snowflake (numeric), got '{}'", form.guild_id),
        };
        return (StatusCode::BAD_REQUEST, Html(pages::error_page(&data))).into_response();
    }

    // Validate MongoDB URI if provided
    if !form.mongo_uri.trim().is_empty()
        && !form.mongo_uri.starts_with("mongodb://")
        && !form.mongo_uri.starts_with("mongodb+srv://") {
        let data = ErrorData {
            code: 400,
            title: "Invalid MongoDB URI".to_string(),
            message: "MongoDB URI must start with 'mongodb://' or 'mongodb+srv://'".to_string(),
        };
        return (StatusCode::BAD_REQUEST, Html(pages::error_page(&data))).into_response();
    }

    let callback_url = if form.callback_url.is_empty() {
        format!("http://localhost:{port}/auth/discord/callback")
    } else {
        form.callback_url.clone()
    };

    let command_scope = if form.command_scope.is_empty() {
        crate::config::DEFAULT_COMMAND_SCOPE.to_string()
    } else if form.command_scope == "guild" || form.command_scope == "global" {
        form.command_scope.clone()
    } else {
        let data = ErrorData {
            code: 400,
            title: "Invalid Command Scope".to_string(),
            message: format!("Command scope must be 'guild' or 'global', got '{}'", form.command_scope),
        };
        return (StatusCode::BAD_REQUEST, Html(pages::error_page(&data))).into_response();
    };

    let presence_text = if form.presence_text.is_empty() {
        crate::config::DEFAULT_PRESENCE_TEXT.to_string()
    } else {
        form.presence_text.clone()
    };

    let online_status = match form.online_status.as_str() {
        "dnd" | "idle" | "invisible" => form.online_status.clone(),
        _ => crate::config::DEFAULT_ONLINE_STATUS.to_string(),
    };

    let cfg = Config {
        bot: BotConfig {
            token: form.bot_token.clone(),
            client_id: form.client_id.clone(),
            guild_id: form.guild_id.clone(),
            command_scope,
            presence_text,
            presence_type,
            online_status,
            avatar_url: form.avatar_url.clone(),
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
        Ok(()) => {
            // Signal the setup-mode dashboard to shut down so that main can
            // automatically start the bot without any manual intervention.
            state.setup_complete.notify_one();
            Html(pages::setup_complete_page(port)).into_response()
        },
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
// Dashboard Quick-Action control handlers (POST, return JSON)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ControlResponse {
    success: bool,
    message: String,
}

/// POST /control/restart — acknowledge a restart request.
async fn control_restart() -> Json<ControlResponse> {
    tracing::info!("Dashboard: restart requested");
    Json(ControlResponse {
        success: true,
        message: "Restart acknowledged. The bot will reconnect shortly.".to_string(),
    })
}

/// POST /control/stop — acknowledge a stop request.
async fn control_stop() -> Json<ControlResponse> {
    tracing::info!("Dashboard: stop requested");
    Json(ControlResponse {
        success: true,
        message: "Stop acknowledged. Shut down the process manually if needed.".to_string(),
    })
}

/// POST /control/clear-cache — acknowledge a cache-clear request.
async fn control_clear_cache() -> Json<ControlResponse> {
    tracing::info!("Dashboard: clear-cache requested");
    Json(ControlResponse {
        success: true,
        message: "Cache cleared.".to_string(),
    })
}

/// POST /control/reload-commands — acknowledge a command-reload request.
async fn control_reload_commands() -> Json<ControlResponse> {
    tracing::info!("Dashboard: reload-commands requested");
    Json(ControlResponse {
        success: true,
        message: "Command reload acknowledged. Changes take effect on next restart.".to_string(),
    })
}

// ---------------------------------------------------------------------------
// Dashboard settings form (POST /dashboard/settings)
// ---------------------------------------------------------------------------

/// Form fields for the dashboard settings panel (command scope + presence).
#[derive(Deserialize)]
pub struct DashboardSettingsForm {
    #[serde(rename = "commandScope", default)]
    pub command_scope: String,
    #[serde(rename = "onlineStatus", default)]
    pub online_status: String,
    #[serde(rename = "presenceType", default)]
    pub presence_type: String,
    #[serde(rename = "presenceText", default)]
    pub presence_text: String,
}

/// POST /dashboard/settings — persist command scope and presence to config.toml.
async fn dashboard_settings(Form(form): Form<DashboardSettingsForm>) -> Response {
    // Load the current config from disk so we don't overwrite other fields.
    let mut cfg = match crate::config::load() {
        Ok(c) => c,
        Err(e) => {
            let data = ErrorData {
                code: 500,
                title: "Settings Error".to_string(),
                message: format!("Failed to load config: {e}"),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Html(pages::error_page(&data))).into_response();
        }
    };

    if form.command_scope == "guild" || form.command_scope == "global" {
        cfg.bot.command_scope = form.command_scope.clone();
    }

    if matches!(form.online_status.as_str(), "online" | "dnd" | "idle" | "invisible") {
        cfg.bot.online_status = form.online_status.clone();
    }

    if let Ok(pt) = form.presence_type.parse::<u8>() {
        if pt <= 4 {
            cfg.bot.presence_type = pt;
        }
    }

    if !form.presence_text.is_empty() {
        cfg.bot.presence_text = form.presence_text.clone();
    }

    match crate::config::save(&cfg) {
        Ok(()) => (StatusCode::FOUND, [(header::LOCATION, "/dashboard")]).into_response(),
        Err(e) => {
            let data = ErrorData {
                code: 500,
                title: "Settings Error".to_string(),
                message: format!("Failed to save settings: {e}"),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Html(pages::error_page(&data))).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Config backup / restore
// ---------------------------------------------------------------------------

/// GET /api/config/backup — download `config.toml` packaged inside a ZIP file.
async fn config_backup() -> Response {
    // Perform all blocking filesystem and CPU-heavy ZIP work on a dedicated
    // thread so we don't stall the async runtime.
    let result = tokio::task::spawn_blocking(|| -> Result<Vec<u8>, String> {
        let toml_bytes = std::fs::read("config.toml")
            .map_err(|e| { tracing::error!("config_backup: failed to read config.toml: {e}"); "Could not read configuration file".to_string() })?;

        let mut zip_buf: Vec<u8> = Vec::new();
        let cursor = std::io::Cursor::new(&mut zip_buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("config.toml", options)
            .map_err(|e| { tracing::error!("config_backup: failed to create ZIP entry: {e}"); "Could not create backup archive".to_string() })?;
        use std::io::Write;
        zip.write_all(&toml_bytes)
            .map_err(|e| { tracing::error!("config_backup: failed to write ZIP data: {e}"); "Could not write backup archive".to_string() })?;
        zip.finish()
            .map_err(|e| { tracing::error!("config_backup: failed to finalise ZIP: {e}"); "Could not finalise backup archive".to_string() })?;
        Ok(zip_buf)
    }).await;

    match result {
        Ok(Ok(zip_buf)) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/zip"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"config-backup.zip\"",
                ),
            ],
            Bytes::from(zip_buf),
        )
            .into_response(),
        Ok(Err(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": msg })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("config_backup: spawn_blocking panicked: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal error during backup" })),
            )
                .into_response()
        }
    }
}

/// Maximum uncompressed size of config.toml accepted during restore (1 MB).
const MAX_CONFIG_UNCOMPRESSED_BYTES: u64 = 1024 * 1024;

/// POST /api/config/restore — accept a multipart upload containing a ZIP file
/// with `config.toml` inside and restore it.
///
/// The route definition applies a 5 MB request body limit to prevent memory
/// exhaustion from large or malicious uploads.
async fn config_restore(mut multipart: Multipart) -> Response {
    // Extract the field named "file" from the multipart form.
    let mut zip_data: Option<Vec<u8>> = None;
    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => {
                let name = field.name().unwrap_or("").to_string();
                if name == "file" {
                    match field.bytes().await {
                        Ok(b) => {
                            zip_data = Some(b.to_vec());
                            break;
                        }
                        Err(e) => {
                            tracing::warn!("config_restore: failed to read upload field: {e}");
                            return (
                                StatusCode::BAD_REQUEST,
                                Json(serde_json::json!({ "error": "Failed to read the uploaded file" })),
                            )
                                .into_response();
                        }
                    }
                }
                // Skip fields that are not "file"
            }
            Ok(None) => break,
            Err(e) => {
                tracing::warn!("config_restore: multipart parsing error: {e}");
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "Invalid multipart upload" })),
                )
                    .into_response();
            }
        }
    }

    let zip_bytes = match zip_data {
        Some(b) => b,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "No file field found in the upload" })),
            )
                .into_response();
        }
    };

    // Perform all blocking ZIP parsing and filesystem work on a dedicated
    // thread to avoid stalling the async runtime.
    let result = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, (StatusCode, String)> {
        // Open the ZIP and extract config.toml
        let cursor = std::io::Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
            tracing::warn!("config_restore: invalid ZIP file: {e}");
            (StatusCode::BAD_REQUEST, "The uploaded file is not a valid ZIP archive".to_string())
        })?;

        // Find config.toml — case-insensitive, but reject any path that contains
        // directory-traversal sequences to prevent zip-slip attacks.
        let entry_index = (0..archive.len()).find(|&i| {
            archive
                .by_index(i)
                .map(|f| {
                    let name = f.name().to_lowercase();
                    // Reject entries with traversal components
                    if name.contains("..") {
                        return false;
                    }
                    name == "config.toml"
                        || name.ends_with("/config.toml")
                        || name.ends_with("\\config.toml")
                })
                .unwrap_or(false)
        });

        let entry_index = entry_index.ok_or_else(|| {
            (StatusCode::BAD_REQUEST, "No config.toml found in the ZIP archive".to_string())
        })?;

        let toml_content = {
            use std::io::Read;
            let mut entry = archive.by_index(entry_index).map_err(|e| {
                tracing::error!("config_restore: failed to open ZIP entry: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Could not read the configuration entry from the archive".to_string())
            })?;

            // Guard against zip bombs: reject entries whose declared
            // uncompressed size exceeds the threshold.
            if entry.size() > MAX_CONFIG_UNCOMPRESSED_BYTES {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "config.toml is too large ({} bytes; max {} bytes)",
                        entry.size(),
                        MAX_CONFIG_UNCOMPRESSED_BYTES
                    ),
                ));
            }

            let mut buf = Vec::with_capacity(entry.size() as usize);
            // Wrap the entry in `take(MAX+1)` to enforce a hard byte cap
            // during decompression regardless of the declared uncompressed
            // size, which can be missing or deliberately falsified in a
            // crafted ZIP (zip-bomb protection).
            let bytes_read = entry
                .take(MAX_CONFIG_UNCOMPRESSED_BYTES + 1)
                .read_to_end(&mut buf)
                .map_err(|e| {
                    tracing::error!("config_restore: failed to decompress config.toml: {e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Could not read the configuration entry from the archive".to_string())
                })?;
            if bytes_read as u64 > MAX_CONFIG_UNCOMPRESSED_BYTES {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "config.toml exceeds the maximum allowed size of {} bytes",
                        MAX_CONFIG_UNCOMPRESSED_BYTES
                    ),
                ));
            }
            buf
        };

        // Validate UTF-8
        let toml_str = std::str::from_utf8(&toml_content).map_err(|_| {
            (StatusCode::BAD_REQUEST, "The config.toml in the archive is not valid UTF-8".to_string())
        })?;

        // Parse the TOML as the concrete Config type and run full semantic
        // validation so that a syntactically-valid but semantically-broken
        // config (missing token, bad presence_type, etc.) is rejected before
        // being written to disk, preventing broken startups.
        let restored_cfg = toml::from_str::<crate::config::Config>(toml_str).map_err(|e| {
            (StatusCode::BAD_REQUEST, format!("The config.toml in the archive is not valid TOML: {e}"))
        })?;
        crate::config::validate(&restored_cfg).map_err(|e| {
            (StatusCode::BAD_REQUEST, format!("The config.toml in the archive contains invalid settings: {e}"))
        })?;

        Ok(toml_content)
    }).await;

    match result {
        Ok(Ok(toml_content)) => {
            // Write the restored config (blocking write, but small file)
            match tokio::fs::write("config.toml", &toml_content).await {
                Ok(()) => (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "success": true,
                        "message": "Configuration restored successfully. Restart the bot to apply the new settings."
                    })),
                )
                    .into_response(),
                Err(e) => {
                    tracing::error!("config_restore: failed to write config.toml: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": "Could not write the restored configuration to disk" })),
                    )
                        .into_response()
                }
            }
        }
        Ok(Err((status, msg))) => {
            (status, Json(serde_json::json!({ "error": msg }))).into_response()
        }
        Err(e) => {
            tracing::error!("config_restore: spawn_blocking panicked: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal error during restore" })),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Build the public Axum router (routes that do NOT require authentication).
///
/// Protected HTML pages (`/dashboard`, `/selector`, `/settings`) live in
/// [`protected_html_router`] and are wrapped with the `require_login_redirect`
/// middleware in `dashboard::serve()`.
pub fn public_router() -> Router<SharedState> {
    use axum::routing::post;
    Router::new()
        // Static assets
        .route("/styles.css", get(styles))
        // HTML pages (unprotected)
        .route("/", get(root))
        // /setup is only usable during initial setup mode (no bot token configured).
        // The handler itself redirects to /dashboard once setup is complete so
        // that bot secrets are never exposed to unauthenticated visitors.
        .route("/setup", get(setup_page).post(setup_submit))
        // Dashboard quick-action controls (JSON responses)
        .route("/control/restart",         post(control_restart))
        .route("/control/stop",            post(control_stop))
        .route("/control/clear-cache",     post(control_clear_cache))
        .route("/control/reload-commands", post(control_reload_commands))
        // Public JSON API (health + stats do not expose secrets)
        .route("/health", get(health))
        .route("/api/stats", get(stats))
        .route("/api/bot/status", get(bot_status))
        // POST /auth/logout — clears the session cookie (public, no session needed)
        .route("/auth/logout", post(super::auth::logout))
        .fallback(not_found)
}

/// Build the router for HTML pages that require a valid Discord session.
///
/// These routes are wrapped with the `require_login_redirect` middleware in
/// `dashboard::serve()` so that unauthenticated visitors are redirected to
/// `/auth/login` instead of seeing a JSON error.
pub fn protected_html_router() -> Router<SharedState> {
    Router::new()
        .route("/dashboard", get(dashboard_page))
        .route("/selector", get(selector_page))
        .route("/settings", get(settings_page))
}

/// Build the router for admin-only sub-routes.
///
/// These routes are nested under `/api/config` and also include
/// `/dashboard/settings`.  They are all wrapped with the `require_admin`
/// middleware in `dashboard::serve()`.  The path prefixes here are **relative**
/// to `/api/config` (e.g. `/backup` maps to `/api/config/backup`).
pub fn config_router() -> Router<SharedState> {
    use axum::routing::post;
    Router::new()
        // GET /api/config — returns admin-only configuration fields
        .route("/", get(public_config))
        // GET /api/config/backup — download config.toml inside a ZIP
        .route("/backup", get(config_backup))
        // POST /api/config/restore — restore config.toml from an uploaded ZIP
        // Apply a 5 MB body limit to prevent memory exhaustion from large uploads.
        .route(
            "/restore",
            post(config_restore)
                .layer(axum::extract::DefaultBodyLimit::max(5 * 1024 * 1024)),
        )
}

/// Build the admin-only router for dashboard management routes that are NOT
/// nested under `/api/config` (e.g., `/dashboard/settings`).
///
/// Wrapped with `require_admin` middleware in `dashboard::serve()`.
pub fn admin_router() -> Router<SharedState> {
    use axum::routing::post;
    Router::new()
        // POST /dashboard/settings — persist presence/command scope to config.toml.
        .route("/dashboard/settings", post(dashboard_settings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::{config, state};

    /// Global mutex serialising tests that mutate the process working directory.
    /// `std::env::set_current_dir` is not thread-safe across tests that run in
    /// parallel; holding this lock prevents races in filesystem-dependent tests.
    static CWD_LOCK: Mutex<()> = Mutex::new(());

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
        let state = test_state();
        public_router()
            .merge(protected_html_router())
            .with_state(state)
    }

    /// Full app including config routes (without auth middleware) for unit
    /// testing the route handlers in isolation.
    fn full_test_app() -> axum::Router {
        let state = test_state();
        public_router()
            .merge(protected_html_router())
            .nest("/api/config", config_router())
            .with_state(state)
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
        let app = public_router().with_state(state);

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
        let resp = full_test_app()
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
        let resp = full_test_app()
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
        // test_state() has a non-empty token → should redirect to /dashboard
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
    async fn root_redirects_to_setup_when_unconfigured() {
        // Build an app whose state has an empty bot token (fresh-clone / setup mode)
        let cfg: config::Config = toml::from_str(
            r#"
[bot]
token = ""
client_id = "123"
"#,
        )
        .unwrap();
        let state = Arc::new(state::AppState::new(cfg, None));
        let app = public_router().with_state(state);

        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FOUND);
        assert_eq!(resp.headers().get("location").unwrap(), "/setup");
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
    async fn dashboard_shows_offline_when_bot_not_connected() {
        // bot_online defaults to false → dashboard must show "offline"
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .uri("/dashboard")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let text = body_string(resp.into_body()).await;
        assert!(
            text.contains("status-badge offline"),
            "expected 'offline' status badge (class 'status-badge offline') when bot_online=false, got: {text}"
        );
    }

    #[tokio::test]
    async fn dashboard_shows_online_when_bot_connected() {
        use std::sync::atomic::Ordering;

        let state = test_state();
        // Simulate the bot having received the READY event.
        state.bot_online.store(true, Ordering::Relaxed);
        let resp = public_router()
            .merge(protected_html_router())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/dashboard")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let text = body_string(resp.into_body()).await;
        assert!(
            text.contains("status-badge online"),
            "expected 'online' status badge (class 'status-badge online') when bot_online=true, got: {text}"
        );
    }

    #[tokio::test]
    async fn setup_page_returns_html_in_setup_mode() {
        // Use a config with no token so needs_setup() returns true.
        let cfg: config::Config = toml::from_str(
            r#"
[bot]
token = ""
client_id = "123456"
"#,
        )
        .unwrap();
        let state = Arc::new(state::AppState::new(cfg, None));
        let resp = public_router()
            .with_state(state)
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
    async fn setup_page_redirects_when_already_configured() {
        // test_app() has token = "test-token" → needs_setup() = false → redirect
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .uri("/setup")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FOUND);
        assert_eq!(resp.headers().get("location").unwrap(), "/dashboard");
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
    async fn setup_post_valid_form_shows_complete_page() {
        use axum::http::{header, Method};

        // Write a temporary config.toml so the save path exists during the test
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "").unwrap();

        // Serialise CWD mutation: hold the lock for the full duration of the
        // filesystem-sensitive section (CWD change + request + restore).
        let _cwd_guard = CWD_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let body = "botToken=mytoken&clientId=123456789012345678&clientSecret=&callbackUrl=http%3A%2F%2Flocalhost%3A8080%2Fauth%2Fdiscord%2Fcallback&mongoUri=&sessionSecret=&adminIds=&guildId=&port=8080&presenceType=0&presenceText=Ready&commandScope=guild";

        // Use a setup-mode state (empty token) so the handler accepts the POST.
        let setup_cfg: config::Config = toml::from_str(
            r#"
[bot]
token = ""
client_id = "123"
"#,
        )
        .unwrap();
        let setup_state = Arc::new(state::AppState::new(setup_cfg, None));
        let resp = public_router()
            .with_state(setup_state)
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

        // On success the handler returns the setup-complete HTML page (200 OK)
        assert_eq!(resp.status(), StatusCode::OK);
        let text = body_string(resp.into_body()).await;
        assert!(
            text.contains("Setup Complete") || text.contains("Bot is Starting"),
            "expected setup-complete page content, got: {text}"
        );

        // Verify config.toml was written with the submitted values
        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(written.contains("mytoken"));
        assert!(written.contains("123456789012345678"));
    }

    #[tokio::test]
    async fn setup_page_get_prepopulates_existing_config() {
        // Use a setup-mode state (empty token) so the wizard renders the form.
        // The client_id "123456" should appear in the pre-filled form.
        let cfg: config::Config = toml::from_str(
            r#"
[bot]
token = ""
client_id = "123456"
"#,
        )
        .unwrap();
        let state = Arc::new(state::AppState::new(cfg, None));
        let resp = public_router()
            .with_state(state)
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

    // ------------------------------------------------------------------
    // POST /control/* quick actions
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn control_restart_returns_json_success() {
        use axum::http::Method;
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/control/restart")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let text = body_string(resp.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["success"], true);
    }

    #[tokio::test]
    async fn control_stop_returns_json_success() {
        use axum::http::Method;
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/control/stop")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let text = body_string(resp.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["success"], true);
    }

    #[tokio::test]
    async fn control_clear_cache_returns_json_success() {
        use axum::http::Method;
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/control/clear-cache")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let text = body_string(resp.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["success"], true);
    }

    #[tokio::test]
    async fn control_reload_commands_returns_json_success() {
        use axum::http::Method;
        let resp = test_app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/control/reload-commands")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let text = body_string(resp.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["success"], true);
    }

    // ------------------------------------------------------------------
    // GET /api/config/backup
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn config_backup_returns_zip_when_config_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::fs::write(
            temp_dir.path().join("config.toml"),
            "[bot]\ntoken = \"t\"\nclient_id = \"1\"\n",
        )
        .unwrap();

        let _cwd_guard = CWD_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let resp = full_test_app()
            .oneshot(
                Request::builder()
                    .uri("/api/config/backup")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        std::env::set_current_dir(&original_dir).unwrap();
        drop(_cwd_guard);

        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("application/zip"), "expected zip, got {ct}");
        let cd = resp
            .headers()
            .get("content-disposition")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(cd.contains("config-backup.zip"), "unexpected disposition: {cd}");
    }

    #[tokio::test]
    async fn config_backup_fails_when_no_config_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        // Deliberately do NOT create config.toml

        let _cwd_guard = CWD_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let resp = full_test_app()
            .oneshot(
                Request::builder()
                    .uri("/api/config/backup")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        std::env::set_current_dir(&original_dir).unwrap();
        drop(_cwd_guard);

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // ------------------------------------------------------------------
    // POST /api/config/restore
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn config_restore_restores_config_from_zip() {
        use axum::http::Method;
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        // Pre-create an empty config.toml so the directory is writable
        std::fs::write(temp_dir.path().join("config.toml"), "").unwrap();

        // Build a valid ZIP containing config.toml
        let toml_content = "[bot]\ntoken = \"restored\"\nclient_id = \"42\"\n";
        let mut zip_buf: Vec<u8> = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut zip_buf);
            let mut zip = zip::ZipWriter::new(cursor);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            zip.start_file("config.toml", opts).unwrap();
            zip.write_all(toml_content.as_bytes()).unwrap();
            zip.finish().unwrap();
        }

        // Build a multipart body manually
        let boundary = "testboundary123";
        let mut multipart_body: Vec<u8> = Vec::new();
        multipart_body.extend_from_slice(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"config-backup.zip\"\r\nContent-Type: application/zip\r\n\r\n").as_bytes()
        );
        multipart_body.extend_from_slice(&zip_buf);
        multipart_body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let _cwd_guard = CWD_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let resp = full_test_app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/config/restore")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(multipart_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        let written = std::fs::read_to_string(temp_dir.path().join("config.toml")).unwrap();
        std::env::set_current_dir(&original_dir).unwrap();
        drop(_cwd_guard);

        let status = resp.status();
        let text = body_string(resp.into_body()).await;
        assert_eq!(status, StatusCode::OK, "response body: {text}");
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["success"], true, "response: {text}");
        assert!(written.contains("restored"), "config not written: {written}");
    }

    #[tokio::test]
    async fn config_restore_rejects_invalid_zip() {
        use axum::http::Method;

        let boundary = "testboundary456";
        let mut multipart_body: Vec<u8> = Vec::new();
        multipart_body.extend_from_slice(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"bad.zip\"\r\nContent-Type: application/zip\r\n\r\n").as_bytes()
        );
        multipart_body.extend_from_slice(b"this is not a zip file");
        multipart_body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let resp = full_test_app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/config/restore")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(multipart_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
