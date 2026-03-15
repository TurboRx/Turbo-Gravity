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
/// The server always binds to `0.0.0.0:{port}` (required for container
/// environments and deployments behind reverse proxies).  The `/api/config/*`
/// routes and `/dashboard/settings` are protected by the `require_admin`
/// middleware which enforces a valid Discord OAuth2 session belonging to
/// `ADMIN_DISCORD_ID`.
pub async fn serve(state: SharedState) -> anyhow::Result<()> {
    let port = state.config.dashboard.port;

    // Always bind to all interfaces so the dashboard is reachable inside
    // containers and behind reverse proxies.
    let bind_addr = format!("0.0.0.0:{port}");

    // Build protected sub-routers.
    // require_admin validates the session cookie and ensures only the
    // configured ADMIN_DISCORD_ID can reach sensitive API routes.
    let admin_guard = axum::middleware::from_fn_with_state(Arc::clone(&state), auth::require_admin);
    // require_login_redirect protects HTML pages: redirects unauthenticated
    // visitors to /auth/login rather than returning a JSON error.
    let login_guard = axum::middleware::from_fn_with_state(Arc::clone(&state), auth::require_login_redirect);

    let config_routes = routes::config_router()
        .route_layer(admin_guard.clone());
    let admin_routes = routes::admin_router()
        .route_layer(admin_guard);
    let protected_pages = routes::protected_html_router()
        .route_layer(login_guard);

    let app = Router::new()
        // Public routes (setup wizard, health, control, etc.)
        .merge(routes::public_router())
        // Protected HTML pages (/dashboard, /selector, /settings)
        .merge(protected_pages)
        // Protected /api/config/* routes (backup, restore, config view)
        .nest("/api/config", config_routes)
        // Protected admin routes (/dashboard/settings, etc.)
        .merge(admin_routes)
        // Discord OAuth2 flow
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        // Allow CORS requests from local HTTP origins (localhost/127.0.0.1).
        // This is mainly for local development where the UI is served from a
        // different HTTP port; in production/cloud same-origin setups, the
        // browser bypasses CORS entirely, so this layer only affects cross-origin callers.
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
        .layer(axum::middleware::from_fn(add_security_headers))
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

/// Middleware that adds standard security headers to all outgoing responses.
///
/// These headers provide defense-in-depth against common web vulnerabilities
/// like XSS, clickjacking, and MIME-sniffing.
async fn add_security_headers(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Content-Security-Policy: restrict where resources can be loaded from.
    // - 'self': allow resources from the same origin.
    // - 'unsafe-inline': required for the embedded theme and dashboard scripts.
    // - fonts.googleapis.com / fonts.gstatic.com: Google Fonts.
    // - cdn.discordapp.com: Discord guild icons.
    let csp = "default-src 'self'; \
               script-src 'self' 'unsafe-inline'; \
               style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
               font-src 'self' https://fonts.gstatic.com; \
               img-src 'self' https://cdn.discordapp.com data:; \
               frame-ancestors 'none'; \
               base-uri 'self'; \
               form-action 'self';";

    headers.insert(
        axum::http::header::CONTENT_SECURITY_POLICY,
        axum::http::HeaderValue::from_static(csp),
    );

    // X-Content-Type-Options: prevent MIME-type sniffing.
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        axum::http::HeaderValue::from_static("nosniff"),
    );

    // X-Frame-Options: prevent clickjacking by disallowing embedding in iframes.
    headers.insert(
        axum::http::header::X_FRAME_OPTIONS,
        axum::http::HeaderValue::from_static("DENY"),
    );

    // Referrer-Policy: control how much referrer information is sent.
    headers.insert(
        axum::http::header::REFERRER_POLICY,
        axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Strict-Transport-Security: force HTTPS (ignored by browsers on http://localhost).
    headers.insert(
        axum::http::header::STRICT_TRANSPORT_SECURITY,
        axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // X-XSS-Protection: legacy header to enable browser XSS filtering.
    headers.insert(
        axum::http::header::HeaderName::from_static("x-xss-protection"),
        axum::http::HeaderValue::from_static("1; mode=block"),
    );

    response
}
