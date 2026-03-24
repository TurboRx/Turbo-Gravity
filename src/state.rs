use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;

use mongodb::Client as MongoClient;
use tokio::sync::{Mutex, Notify};

use crate::config::Config;

/// Information stored per active dashboard session.
#[derive(Clone)]
pub struct SessionInfo {
    /// Discord user ID (snowflake string).
    pub user_id: String,
    /// Discord username (e.g. "myname").
    pub username: String,
}

/// Application-wide shared state threaded through Poise's `Data` type parameter
/// and cloned into the Axum dashboard router via `Arc`.
pub struct AppState {
    pub config: Config,
    pub db: Option<MongoClient>,
    /// Notified when the setup wizard successfully saves a configuration.
    /// The main entry point listens for this signal to stop the setup-mode
    /// dashboard and automatically start the bot without any manual intervention.
    pub setup_complete: Notify,
    /// Active dashboard sessions: session_id → SessionInfo (user_id + username).
    pub sessions: Mutex<HashMap<String, SessionInfo>>,
    /// Pending OAuth2 CSRF states awaiting callback validation.
    /// The value is the redirect URI used when initiating the login, so the
    /// callback handler can reuse the exact same URI for the token exchange
    /// (required by Discord's OAuth2 flow when the redirect URI is dynamic,
    /// e.g. auto-detected from the Host header on cloud deployments).
    pub oauth_states: Mutex<HashMap<String, String>>,
    /// Whether the Discord gateway connection is currently live.
    /// Set to `true` on READY / Resume; reset to `false` when the connection
    /// drops so the dashboard reflects the real bot status.
    pub bot_online: AtomicBool,
    /// Number of guilds the bot is currently a member of (updated on READY).
    pub guild_count: AtomicUsize,
    /// Instant at which the process started — used to compute bot uptime.
    pub start_time: std::time::Instant,
}

impl AppState {
    pub fn new(config: Config, db: Option<MongoClient>) -> Self {
        Self {
            config,
            db,
            setup_complete: Notify::new(),
            sessions: Mutex::new(HashMap::new()),
            oauth_states: Mutex::new(HashMap::new()),
            bot_online: AtomicBool::new(false),
            guild_count: AtomicUsize::new(0),
            start_time: std::time::Instant::now(),
        }
    }

    /// Returns a reference to the MongoDB database, or `None` if no DB is configured.
    pub fn database(&self) -> Option<mongodb::Database> {
        self.db
            .as_ref()
            .map(|c| c.database("turbo_gravity"))
    }
}

/// Type alias used as the Poise `Data` type parameter throughout the bot.
pub type SharedState = Arc<AppState>;
