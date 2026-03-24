/// Discord OAuth2 authentication for the dashboard.
///
/// Routes:
///   GET  /auth/login    – redirect the browser to Discord's authorization page.
///   GET  /auth/callback – exchange the authorization code for an access token,
///                         fetch the Discord profile, validate the admin ID, and
///                         issue an HttpOnly session cookie.
///
/// Middleware:
///   `require_admin` – verifies a valid session cookie and checks that the
///                     session belongs to the configured ADMIN_DISCORD_ID.
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    RedirectUrl, Scope, TokenUrl,
};
use serde::{Deserialize, Serialize};

use crate::state::{SessionInfo, SharedState};

// ---------------------------------------------------------------------------
// OAuth2 client helpers
// ---------------------------------------------------------------------------

/// Concrete [`BasicClient`] type returned by [`build_oauth_client`].
///
/// The two `EndpointSet` positions correspond to `HasAuthUrl` and `HasTokenUrl`
/// being set via [`BasicClient::set_auth_uri`] and [`BasicClient::set_token_uri`].
type DiscordOauthClient = BasicClient<
    EndpointSet,    // HasAuthUrl
    EndpointNotSet, // HasDeviceAuthUrl
    EndpointNotSet, // HasIntrospectionUrl
    EndpointNotSet, // HasRevocationUrl
    EndpointSet,    // HasTokenUrl
>;

/// Build a Discord OAuth2 [`BasicClient`] using the supplied `redirect_uri`.
///
/// The `client_id` and `client_secret` are read from env vars first
/// (`DISCORD_CLIENT_ID` / `DISCORD_CLIENT_SECRET`), falling back to the values
/// stored in the application config.
fn build_oauth_client(
    state: &crate::state::AppState,
    redirect_uri: &str,
) -> anyhow::Result<DiscordOauthClient> {
    let client_id =
        std::env::var("DISCORD_CLIENT_ID").unwrap_or_else(|_| state.config.bot.client_id.clone());
    let client_secret = std::env::var("DISCORD_CLIENT_SECRET")
        .unwrap_or_else(|_| state.config.dashboard.client_secret.clone());

    anyhow::ensure!(!client_id.is_empty(), "DISCORD_CLIENT_ID is not configured");
    anyhow::ensure!(
        !client_secret.is_empty(),
        "DISCORD_CLIENT_SECRET is not configured"
    );
    anyhow::ensure!(
        !redirect_uri.is_empty(),
        "DISCORD_REDIRECT_URI is not configured"
    );

    let client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_uri(AuthUrl::new(
            "https://discord.com/api/oauth2/authorize".to_string(),
        )?)
        .set_token_uri(TokenUrl::new(
            "https://discord.com/api/oauth2/token".to_string(),
        )?)
        .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?);

    Ok(client)
}

/// Determine the OAuth2 redirect URI for the current request.
///
/// Resolution order (first non-empty value wins):
/// 1. `DISCORD_REDIRECT_URI` environment variable – explicit override, highest priority.
/// 2. Auto-detection from reverse-proxy headers (`X-Forwarded-Proto` + `X-Forwarded-Host`
///    or `Host`) – only performed when `X-Forwarded-Proto` is present and carries a
///    valid `http` or `https` scheme, avoiding a spurious `http://` URL when the
///    header is absent.  `X-Forwarded-Host` is preferred over `Host` because some
///    reverse proxies rewrite the `Host` header while forwarding the original in
///    `X-Forwarded-Host`.
/// 3. `config.dashboard.callback_url` – static fallback for bare-metal / local setups.
fn detect_redirect_uri(state: &crate::state::AppState, headers: &axum::http::HeaderMap) -> String {
    // 1. Explicit env-var override.
    if let Ok(uri) = std::env::var("DISCORD_REDIRECT_URI") {
        if !uri.is_empty() {
            return uri;
        }
    }

    // 2. Auto-detect from reverse-proxy headers.
    //    Only proceed when X-Forwarded-Proto is present and specifies a valid
    //    scheme; otherwise fall through to the configured callback URL to avoid
    //    constructing an incorrect `http://` redirect when no proxy is involved.
    //    X-Forwarded-Proto may carry a comma-separated list when there are
    //    multiple proxies in the chain; take only the first (outermost) value.
    if let Some(proto_header) = headers.get("x-forwarded-proto") {
        if let Ok(proto_str) = proto_header.to_str() {
            if let Some(first) = proto_str.split(',').next() {
                let scheme = first.trim().to_ascii_lowercase();
                if scheme == "http" || scheme == "https" {
                    // Prefer X-Forwarded-Host (the original host as seen by the
                    // outermost proxy); fall back to the Host header.
                    let host = headers
                        .get("x-forwarded-host")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .or_else(|| {
                            headers
                                .get("host")
                                .and_then(|v| v.to_str().ok())
                                .map(|s| s.trim())
                                .filter(|s| !s.is_empty())
                        });

                    if let Some(host) = host {
                        return format!("{scheme}://{host}/auth/callback");
                    }
                }
            }
        }
    }

    // 3. Static config fallback.
    state.config.dashboard.callback_url.clone()
}

// ---------------------------------------------------------------------------
// /auth/login
// ---------------------------------------------------------------------------

/// `GET /auth/login` – generate a Discord authorization URL and redirect the
/// browser to it.  A CSRF state token is stored in server memory so that the
/// callback handler can verify it.
///
/// The OAuth2 redirect URI is resolved dynamically from request headers so that
/// the dashboard works correctly on cloud deployments without
/// requiring manual configuration of `DISCORD_REDIRECT_URI`.
pub async fn login(State(state): State<SharedState>, headers: axum::http::HeaderMap) -> Response {
    let redirect_uri = detect_redirect_uri(&state, &headers);

    let client = match build_oauth_client(&state, &redirect_uri) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("OAuth2 client configuration error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "OAuth2 is not configured. Set DISCORD_CLIENT_ID, DISCORD_CLIENT_SECRET, and DISCORD_REDIRECT_URI.",
            )
                .into_response();
        }
    };

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .url();

    // Persist the CSRF state along with the redirect URI so the callback
    // handler can perform the token exchange with the exact same URI.
    {
        let mut states = state.oauth_states.lock().await;
        // Evict one arbitrary entry to prevent unbounded growth without
        // invalidating all in-flight login attempts.
        if states.len() >= 256 {
            if let Some(evicted_key) = states.keys().next().cloned() {
                states.remove(&evicted_key);
            }
        }
        states.insert(csrf_token.secret().clone(), redirect_uri);
    }

    tracing::info!("OAuth2 login initiated, redirecting to Discord");
    Redirect::temporary(auth_url.as_str()).into_response()
}

// ---------------------------------------------------------------------------
// /auth/callback
// ---------------------------------------------------------------------------

/// Query parameters sent by Discord to the redirect URI.
#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

/// Partial token response returned by Discord's `/oauth2/token` endpoint.
#[derive(Deserialize)]
struct DiscordTokenResponse {
    access_token: String,
}

/// Discord user profile returned by `GET /users/@me`.
#[derive(Deserialize, Serialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
}

/// `GET /auth/callback` – validate the CSRF state, exchange the authorization
/// code for an access token, fetch the Discord profile, check that the user is
/// the configured admin, and issue a session cookie.
pub async fn callback(
    State(state): State<SharedState>,
    Query(params): Query<CallbackParams>,
) -> Response {
    // --- 1. Validate CSRF state --------------------------------------------
    let redirect_uri = {
        let mut states = state.oauth_states.lock().await;
        states.remove(&params.state)
    };

    let redirect_uri = match redirect_uri {
        Some(uri) => uri,
        None => {
            tracing::warn!("OAuth2 callback received invalid or expired CSRF state");
            return (
                StatusCode::BAD_REQUEST,
                "Invalid or expired CSRF state. Please try logging in again.",
            )
                .into_response();
        }
    };

    // --- 2. Build OAuth2 client --------------------------------------------
    let oauth_client = match build_oauth_client(&state, &redirect_uri) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("OAuth2 client configuration error during callback: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "OAuth2 is not configured",
            )
                .into_response();
        }
    };

    // --- 3. Exchange authorization code for access token -------------------
    // We perform the token exchange manually using reqwest so that we do not
    // depend on the oauth2 crate's built-in HTTP backend and its transitive
    // reqwest/TLS dependencies.
    let client_id = oauth_client.client_id().as_str().to_owned();
    let client_secret_val = std::env::var("DISCORD_CLIENT_SECRET")
        .unwrap_or_else(|_| state.config.dashboard.client_secret.clone());

    let http = reqwest::Client::new();
    let token_resp = match http
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret_val.as_str()),
            ("grant_type", "authorization_code"),
            ("code", params.code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Discord token exchange HTTP error: {e}");
            return (
                StatusCode::BAD_GATEWAY,
                "Failed to contact Discord's token endpoint",
            )
                .into_response();
        }
    };

    if !token_resp.status().is_success() {
        let status = token_resp.status();
        let body = token_resp.text().await.unwrap_or_default();
        tracing::error!("Discord token exchange failed ({status}): {body}");
        return (
            StatusCode::UNAUTHORIZED,
            "Discord rejected the authorization code",
        )
            .into_response();
    }

    let token: DiscordTokenResponse = match token_resp.json().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to deserialize Discord token response: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected response from Discord",
            )
                .into_response();
        }
    };

    // --- 4. Fetch Discord user profile -------------------------------------
    let user_resp = match http
        .get("https://discord.com/api/users/@me")
        .bearer_auth(&token.access_token)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Discord user info HTTP error: {e}");
            return (StatusCode::BAD_GATEWAY, "Failed to fetch Discord profile").into_response();
        }
    };

    if !user_resp.status().is_success() {
        let status = user_resp.status();
        let body = user_resp.text().await.unwrap_or_default();
        tracing::error!("Discord user profile request failed ({status}): {body}");
        return (
            StatusCode::UNAUTHORIZED,
            "Discord rejected the access token while fetching user profile",
        )
            .into_response();
    }

    let discord_user: DiscordUser = match user_resp.json().await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Failed to deserialize Discord user profile: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected response from Discord user API",
            )
                .into_response();
        }
    };

    // --- 5. Verify admin identity ------------------------------------------
    let admin_id = std::env::var("ADMIN_DISCORD_ID").unwrap_or_default();
    if admin_id.is_empty() {
        tracing::error!("ADMIN_DISCORD_ID is not set; denying all login attempts");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Dashboard admin is not configured",
        )
            .into_response();
    }
    if discord_user.id != admin_id {
        tracing::warn!(
            "Login denied for Discord user {} ({}): not the configured admin",
            discord_user.username,
            discord_user.id
        );
        return (
            StatusCode::FORBIDDEN,
            "Access denied: your Discord account is not authorized to access this dashboard.",
        )
            .into_response();
    }

    // --- 6. Issue session cookie -------------------------------------------
    let session_id = generate_session_id();
    {
        let mut sessions = state.sessions.lock().await;
        // Evict one entry to prevent unbounded growth without invalidating all
        // active sessions (important: do NOT clear the whole map or the current
        // admin would be logged out immediately after logging in).
        if sessions.len() >= 64 {
            if let Some(oldest) = sessions.keys().next().cloned() {
                sessions.remove(&oldest);
            }
        }
        sessions.insert(
            session_id.clone(),
            SessionInfo {
                user_id: discord_user.id.clone(),
                username: discord_user.username.clone(),
            },
        );
    }

    tracing::info!(
        "Admin {} ({}) logged in successfully",
        discord_user.username,
        discord_user.id
    );

    // HttpOnly prevents JS access; SameSite=Lax prevents most CSRF attacks.
    // Secure ensures the cookie is only sent over HTTPS (important when deployed
    // behind a TLS-terminating reverse proxy).
    let cookie = format!("session_id={session_id}; HttpOnly; Secure; Path=/; SameSite=Lax");
    (
        StatusCode::FOUND,
        [
            (header::LOCATION, "/dashboard".to_string()),
            (header::SET_COOKIE, cookie),
        ],
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Logout handler
// ---------------------------------------------------------------------------

/// `POST /auth/logout` — invalidate the current session and redirect to /auth/login.
pub async fn logout(State(state): State<SharedState>, request: axum::extract::Request) -> Response {
    // Extract session_id from the Cookie header and remove it from the store.
    let session_id = request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| {
            raw.split(';').find_map(|c| {
                let c = c.trim();
                c.strip_prefix("session_id=").map(str::to_string)
            })
        });

    if let Some(sid) = session_id {
        state.sessions.lock().await.remove(&sid);
    }

    // Clear the session cookie and redirect to login.
    // Note: the Secure flag is intentionally kept consistent with the login cookie
    // (set in the callback handler). In non-HTTPS environments the cookie should
    // not have been sent at all, so the Max-Age=0 expiry covers HTTP-only deploys.
    let clear_cookie = "session_id=; HttpOnly; Secure; Path=/; SameSite=Lax; Max-Age=0";
    (
        StatusCode::FOUND,
        [
            (header::LOCATION, "/auth/login".to_string()),
            (header::SET_COOKIE, clear_cookie.to_string()),
        ],
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Admin auth guard middleware
// ---------------------------------------------------------------------------

/// Axum middleware that protects `/api/config/*` routes.
///
/// Reads the `session_id` cookie, looks up the session in [`AppState`], and
/// verifies that the stored Discord user ID matches `ADMIN_DISCORD_ID`.  Any
/// request that fails these checks receives a `401 Unauthorized` or
/// `403 Forbidden` JSON response before reaching the route handler.
pub async fn require_admin(
    State(state): State<SharedState>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    // Extract `session_id` from the Cookie header.
    let session_id = request
        .headers()
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|c| {
                let c = c.trim();
                c.strip_prefix("session_id=").map(str::to_string)
            })
        });

    let session_id = match session_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Authentication required"})),
            )
                .into_response();
        }
    };

    // Look up the session.
    let user_id = {
        let sessions = state.sessions.lock().await;
        sessions.get(&session_id).map(|s| s.user_id.clone())
    };

    let user_id = match user_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid or expired session"})),
            )
                .into_response();
        }
    };

    // Verify the admin ID.
    let admin_id = std::env::var("ADMIN_DISCORD_ID").unwrap_or_default();
    if admin_id.is_empty() {
        tracing::error!(
            "ADMIN_DISCORD_ID is not set; cannot authorise session. \
             Set the ADMIN_DISCORD_ID environment variable to your Discord user ID."
        );
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Dashboard admin is not configured (ADMIN_DISCORD_ID is unset)"})),
        )
            .into_response();
    }
    if user_id != admin_id {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        )
            .into_response();
    }

    next.run(request).await
}

// ---------------------------------------------------------------------------
// Login-redirect middleware (protects HTML dashboard pages)
// ---------------------------------------------------------------------------

/// Middleware that ensures the request has a valid session.
///
/// Unlike [`require_admin`] (which returns JSON errors for API routes), this
/// middleware issues an HTML redirect to `/auth/login` so the browser shows
/// the Discord login page instead of a raw JSON error.
///
/// Call this on routes that serve HTML pages (e.g., `/dashboard`, `/selector`).
pub async fn require_login_redirect(
    State(state): State<SharedState>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    // Extract session_id from cookie.
    let session_id = extract_session_id(request.headers());

    let session_id = match session_id {
        Some(id) => id,
        None => {
            return (StatusCode::FOUND, [(header::LOCATION, "/auth/login")]).into_response();
        }
    };

    let valid = {
        let sessions = state.sessions.lock().await;
        sessions.contains_key(&session_id)
    };

    if !valid {
        return (StatusCode::FOUND, [(header::LOCATION, "/auth/login")]).into_response();
    }

    next.run(request).await
}

/// Extract the [`SessionInfo`] for the current request, or `None` if the
/// session cookie is absent or does not correspond to a known session.
///
/// This is a convenience helper for route handlers that need the logged-in
/// user's information (e.g., to display the username in the topbar).
pub async fn current_session(
    state: &crate::state::AppState,
    headers: &axum::http::HeaderMap,
) -> Option<crate::state::SessionInfo> {
    let session_id = extract_session_id(headers)?;

    state.sessions.lock().await.get(&session_id).cloned()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the `session_id` from the `Cookie` header, if present.
fn extract_session_id(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| {
            raw.split(';').find_map(|c| {
                let c = c.trim();
                c.strip_prefix("session_id=").map(str::to_string)
            })
        })
}

/// Generate a cryptographically random 32-byte session ID, hex-encoded (64 chars).
fn generate_session_id() -> String {
    let mut id = String::with_capacity(64);
    for _ in 0..32 {
        let _ = std::fmt::Write::write_fmt(&mut id, format_args!("{:02x}", rand::random::<u8>()));
    }
    id
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use std::sync::Arc;
    use std::sync::LazyLock;
    use tokio::sync::Mutex as TokioMutex;
    use tower::ServiceExt;

    use crate::{config, state};

    /// Global mutex serialising tests that read or write process environment
    /// variables.  `std::env::set_var` / `remove_var` are not thread-safe across
    /// parallel test threads; holding this lock prevents races.
    static ENV_LOCK: LazyLock<TokioMutex<()>> = LazyLock::new(|| TokioMutex::new(()));

    /// RAII guard that removes a named environment variable when dropped,
    /// ensuring cleanup happens even if the test panics.
    struct EnvVarGuard(&'static str);
    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            std::env::remove_var(self.0);
        }
    }

    fn test_state() -> state::SharedState {
        let cfg: config::Config = toml::from_str(
            r#"
[bot]
token = "test-token"
client_id = "123456"
[dashboard]
enable_dashboard = true
port = 8080
"#,
        )
        .expect("test config must parse");
        Arc::new(state::AppState::new(cfg, None))
    }

    /// A trivial protected handler that always returns 200 OK.
    async fn protected() -> &'static str {
        "secret"
    }

    /// Build a minimal router with `require_admin` guarding `/secret`.
    fn guarded_app(app_state: state::SharedState) -> Router {
        Router::new()
            .route(
                "/secret",
                get(protected).route_layer(axum::middleware::from_fn_with_state(
                    Arc::clone(&app_state),
                    require_admin,
                )),
            )
            .with_state(app_state)
    }

    #[tokio::test]
    async fn require_admin_rejects_missing_cookie() {
        let resp = guarded_app(test_state())
            .oneshot(
                Request::builder()
                    .uri("/secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn require_admin_rejects_unknown_session() {
        let resp = guarded_app(test_state())
            .oneshot(
                Request::builder()
                    .uri("/secret")
                    .header("cookie", "session_id=nonexistent_session")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn require_admin_rejects_non_admin_session() {
        let state = test_state();
        // Register a session for a user who is NOT the admin.
        state.sessions.lock().await.insert(
            "valid_session".to_string(),
            crate::state::SessionInfo {
                user_id: "999999".to_string(),
                username: "testuser".to_string(),
            },
        );
        // ADMIN_DISCORD_ID is not set, so "999999" will never match.
        let resp = guarded_app(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .uri("/secret")
                    .header("cookie", "session_id=valid_session")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Either FORBIDDEN or UNAUTHORIZED depending on whether ADMIN_DISCORD_ID is set.
        // 500 is also valid when ADMIN_DISCORD_ID is not configured at all.
        assert!(
            resp.status() == StatusCode::FORBIDDEN
                || resp.status() == StatusCode::UNAUTHORIZED
                || resp.status() == StatusCode::INTERNAL_SERVER_ERROR,
            "expected 401, 403, or 500, got {}",
            resp.status()
        );
    }

    #[tokio::test]
    async fn require_admin_allows_valid_admin_session() {
        let state = test_state();
        let admin_id = "777777777777777777";
        // Register the session
        state.sessions.lock().await.insert(
            "admin_session".to_string(),
            crate::state::SessionInfo {
                user_id: admin_id.to_string(),
                username: "admin".to_string(),
            },
        );

        // Set the env var so the middleware can verify the admin.
        // Use ENV_LOCK to prevent concurrent tests from seeing a stale value.
        // EnvVarGuard ensures cleanup even if the test panics.
        // NOTE: _lock_guard must be declared BEFORE _env_guard so that it is
        // dropped AFTER _env_guard (Rust drops in reverse declaration order).
        // This ensures the env var is removed while the lock is still held.
        let _lock_guard = ENV_LOCK.lock().await;
        let _env_guard = EnvVarGuard("ADMIN_DISCORD_ID");
        std::env::set_var("ADMIN_DISCORD_ID", admin_id);

        let resp = guarded_app(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .uri("/secret")
                    .header("cookie", "session_id=admin_session")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = body::to_bytes(resp.into_body(), 1024).await.unwrap();
        assert_eq!(&bytes[..], b"secret");
    }

    #[test]
    fn session_id_has_correct_length() {
        let id = generate_session_id();
        // 32 bytes → 64 hex characters
        assert_eq!(id.len(), 64);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn session_ids_are_unique() {
        let a = generate_session_id();
        let b = generate_session_id();
        assert_ne!(a, b, "two generated session IDs must differ");
    }

    // ------------------------------------------------------------------
    // detect_redirect_uri
    // ------------------------------------------------------------------

    fn make_headers(pairs: &[(&str, &str)]) -> axum::http::HeaderMap {
        let mut map = axum::http::HeaderMap::new();
        for (k, v) in pairs {
            map.insert(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                axum::http::HeaderValue::from_str(v).unwrap(),
            );
        }
        map
    }

    #[tokio::test]
    async fn detect_redirect_uri_uses_env_var_override() {
        // DISCORD_REDIRECT_URI env var takes priority over any headers.
        // _lock_guard declared first so it drops LAST (after _env_guard).
        let _lock_guard = ENV_LOCK.lock().await;
        let _env_guard = EnvVarGuard("DISCORD_REDIRECT_URI");
        std::env::set_var(
            "DISCORD_REDIRECT_URI",
            "https://override.example.com/auth/callback",
        );
        let state = test_state();
        let headers = make_headers(&[
            ("x-forwarded-proto", "https"),
            ("host", "other.example.com"),
        ]);
        let uri = detect_redirect_uri(&state, &headers);
        assert_eq!(uri, "https://override.example.com/auth/callback");
    }

    #[tokio::test]
    async fn detect_redirect_uri_auto_detects_from_forwarded_headers() {
        // When DISCORD_REDIRECT_URI is absent but X-Forwarded-Proto + Host are
        // present, the URI should be built from those headers.
        let _lock_guard = ENV_LOCK.lock().await;
        let _env_guard = EnvVarGuard("DISCORD_REDIRECT_URI");
        std::env::remove_var("DISCORD_REDIRECT_URI");
        let state = test_state();
        let headers = make_headers(&[
            ("x-forwarded-proto", "https"),
            ("host", "mybot.example.com"),
        ]);
        let uri = detect_redirect_uri(&state, &headers);
        assert_eq!(uri, "https://mybot.example.com/auth/callback");
    }

    #[tokio::test]
    async fn detect_redirect_uri_prefers_x_forwarded_host_over_host() {
        let _lock_guard = ENV_LOCK.lock().await;
        let _env_guard = EnvVarGuard("DISCORD_REDIRECT_URI");
        std::env::remove_var("DISCORD_REDIRECT_URI");
        let state = test_state();
        let headers = make_headers(&[
            ("x-forwarded-proto", "https"),
            ("x-forwarded-host", "public.example.com"),
            ("host", "internal-hostname"),
        ]);
        let uri = detect_redirect_uri(&state, &headers);
        assert_eq!(uri, "https://public.example.com/auth/callback");
    }

    #[tokio::test]
    async fn detect_redirect_uri_accepts_multi_value_forwarded_proto() {
        // X-Forwarded-Proto may be a comma-separated list; only the first value is used.
        let _lock_guard = ENV_LOCK.lock().await;
        let _env_guard = EnvVarGuard("DISCORD_REDIRECT_URI");
        std::env::remove_var("DISCORD_REDIRECT_URI");
        let state = test_state();
        let headers = make_headers(&[
            ("x-forwarded-proto", "https, http"),
            ("host", "mybot.example.com"),
        ]);
        let uri = detect_redirect_uri(&state, &headers);
        assert_eq!(uri, "https://mybot.example.com/auth/callback");
    }

    #[tokio::test]
    async fn detect_redirect_uri_falls_back_to_config_when_no_forwarded_proto() {
        // Without X-Forwarded-Proto the function must not construct an http://
        // URL from the Host header alone — it must use the configured callback URL.
        let _lock_guard = ENV_LOCK.lock().await;
        let _env_guard = EnvVarGuard("DISCORD_REDIRECT_URI");
        std::env::remove_var("DISCORD_REDIRECT_URI");
        let state = test_state();
        let headers = make_headers(&[("host", "mybot.example.com")]);
        let uri = detect_redirect_uri(&state, &headers);
        // Should be the configured callback_url default, not an invented http:// URL.
        assert_eq!(uri, state.config.dashboard.callback_url);
    }

    #[tokio::test]
    async fn detect_redirect_uri_ignores_unknown_scheme() {
        // An X-Forwarded-Proto value other than http/https must not be used to
        // construct a redirect URI; fall through to config fallback.
        let _lock_guard = ENV_LOCK.lock().await;
        let _env_guard = EnvVarGuard("DISCORD_REDIRECT_URI");
        std::env::remove_var("DISCORD_REDIRECT_URI");
        let state = test_state();
        let headers = make_headers(&[("x-forwarded-proto", "ftp"), ("host", "mybot.example.com")]);
        let uri = detect_redirect_uri(&state, &headers);
        assert_eq!(uri, state.config.dashboard.callback_url);
    }

    #[tokio::test]
    async fn detect_redirect_uri_falls_back_to_config_when_host_empty() {
        // X-Forwarded-Proto is valid but Host is empty — should not build a malformed URI.
        let _lock_guard = ENV_LOCK.lock().await;
        let _env_guard = EnvVarGuard("DISCORD_REDIRECT_URI");
        std::env::remove_var("DISCORD_REDIRECT_URI");
        let state = test_state();
        let headers = make_headers(&[("x-forwarded-proto", "https")]);
        let uri = detect_redirect_uri(&state, &headers);
        assert_eq!(uri, state.config.dashboard.callback_url);
    }
}
