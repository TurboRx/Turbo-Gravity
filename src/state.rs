use std::sync::Arc;

use mongodb::Client as MongoClient;

use crate::config::Config;

/// Application-wide shared state threaded through Poise's `Data` type parameter
/// and cloned into the Axum dashboard router via `Arc`.
pub struct AppState {
    pub config: Config,
    pub db: Option<MongoClient>,
}

impl AppState {
    pub fn new(config: Config, db: Option<MongoClient>) -> Self {
        Self { config, db }
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
