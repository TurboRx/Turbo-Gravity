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

/// Bot configuration loaded from `[bot]` section of `config.toml`.
#[allow(dead_code)] // client_id reserved for future OAuth2 use
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

/// Dashboard / OAuth2 configuration loaded from `[dashboard]` section.
// Fields like session_secret, client_secret, callback_url, and admin_ids
// are intentionally included for future Discord OAuth2 login support.
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_toml() -> &'static str {
        r#"
[bot]
token = "test-token"
client_id = "123456"
"#
    }

    #[test]
    fn parse_minimal_config_applies_defaults() {
        let cfg: Config = toml::from_str(minimal_toml()).unwrap();
        assert_eq!(cfg.bot.token, "test-token");
        assert_eq!(cfg.bot.client_id, "123456");
        // defaults
        assert_eq!(cfg.bot.command_scope, "guild");
        assert_eq!(cfg.bot.presence_text, "Ready to serve");
        assert_eq!(cfg.bot.presence_type, 0);
        assert!(cfg.database.mongo_uri.is_empty());
        assert!(!cfg.dashboard.enable_dashboard);
        assert_eq!(cfg.dashboard.port, 8080);
    }

    #[test]
    fn parse_full_config() {
        let toml = r#"
[bot]
token = "bot-token"
client_id = "app-id"
guild_id = "my-guild"
command_scope = "global"
presence_text = "with Rust"
presence_type = 2

[database]
mongo_uri = "mongodb://localhost:27017"

[dashboard]
enable_dashboard = true
port = 9090
admin_ids = ["111", "222"]
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.bot.guild_id, "my-guild");
        assert_eq!(cfg.bot.command_scope, "global");
        assert_eq!(cfg.bot.presence_type, 2);
        assert_eq!(cfg.database.mongo_uri, "mongodb://localhost:27017");
        assert!(cfg.dashboard.enable_dashboard);
        assert_eq!(cfg.dashboard.port, 9090);
        assert_eq!(cfg.dashboard.admin_ids, vec!["111", "222"]);
    }

    #[test]
    fn missing_required_fields_returns_error() {
        // token and client_id are required with no defaults
        let result: Result<Config, _> = toml::from_str("[bot]\ntoken = \"x\"");
        assert!(result.is_err(), "client_id is required");
    }

    #[test]
    fn dashboard_default_impl_matches_parsed_defaults() {
        let default = DashboardConfig::default();
        assert!(!default.enable_dashboard);
        assert_eq!(default.port, 8080);
        assert_eq!(
            default.callback_url,
            "http://localhost:8080/auth/discord/callback"
        );
    }
}
