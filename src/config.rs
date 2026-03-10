use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

/// Top-level configuration, deserialized from `config.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bot: BotConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub dashboard: DashboardConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    pub token: String,
    pub client_id: String,
    #[serde(default)]
    pub guild_id: String,
    #[serde(default = "default_command_scope")]
    pub command_scope: String,
    #[serde(default = "default_presence_text")]
    pub presence_text: String,
    #[serde(default)]
    pub presence_type: u8,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub mongo_uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DashboardConfig {
    #[serde(default = "default_enable_dashboard")]
    pub enable_dashboard: bool,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub session_secret: String,
    #[serde(default)]
    pub client_secret: String,
    #[serde(default = "default_callback_url")]
    pub callback_url: String,
    #[serde(default)]
    pub admin_ids: Vec<String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enable_dashboard: false,
            port: 8080,
            session_secret: String::new(),
            client_secret: String::new(),
            callback_url: "http://localhost:8080/auth/discord/callback".into(),
            admin_ids: Vec::new(),
        }
    }
}

fn default_command_scope() -> String {
    "guild".into()
}

fn default_presence_text() -> String {
    "Ready to serve".into()
}

fn default_enable_dashboard() -> bool {
    true
}

fn default_port() -> u16 {
    8080
}

fn default_callback_url() -> String {
    "http://localhost:8080/auth/discord/callback".into()
}

/// Load `config.toml` from the current working directory.
pub fn load() -> anyhow::Result<Config> {
    let path = Path::new("config.toml");
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file at '{}'", path.display()))?;
    let cfg: Config = toml::from_str(&raw).context("Failed to parse config.toml")?;
    Ok(cfg)
}
