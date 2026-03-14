use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use mongodb::Client as MongoClient;
use tokio::sync::{Mutex, Notify};

use crate::config::Config;

/// Application-wide shared state threaded through Poise's `Data` type parameter
/// and cloned into the Axum dashboard router via `Arc`.
pub struct AppState {
    pub config: Config,
    pub db: Option<MongoClient>,
    /// Notified when the setup wizard successfully saves a configuration.
    /// The main entry point listens for this signal to stop the setup-mode
    /// dashboard and automatically start the bot.
    pub setup_complete: Notify,
    /// Active dashboard sessions: session_id → Discord user ID.
    pub sessions: Mutex<HashMap<String, String>>,
    /// Pending OAuth2 CSRF states awaiting callback validation.
    pub oauth_states: Mutex<HashMap<String, ()>>,
    /// Whether the Discord gateway connection is currently live.
    /// Set to `true` on READY / Resume; reset to `false` when the connection
    /// drops so the dashboard reflects the real bot status.
    pub bot_online: AtomicBool,
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
