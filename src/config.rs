use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Top-level configuration, deserialized from `config.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub bot: BotConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub dashboard: DashboardConfig,
}

/// Bot configuration loaded from `[bot]` section of `config.toml`.
#[allow(dead_code)] // client_id reserved for future OAuth2 use
#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub mongo_uri: String,
}

/// Dashboard / OAuth2 configuration loaded from `[dashboard]` section.
// Fields like session_secret, client_secret, callback_url, and admin_ids
// are intentionally included for future Discord OAuth2 login support.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
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
            port: DEFAULT_PORT,
            session_secret: String::new(),
            client_secret: String::new(),
            callback_url: DEFAULT_CALLBACK_URL.into(),
            admin_ids: Vec::new(),
        }
    }
}

pub const DEFAULT_COMMAND_SCOPE: &str = "guild";
pub const DEFAULT_PRESENCE_TEXT: &str = "Ready to serve";
pub const DEFAULT_CALLBACK_URL: &str = "http://localhost:8080/auth/discord/callback";
pub const DEFAULT_PORT: u16 = 8080;

fn default_command_scope() -> String {
    DEFAULT_COMMAND_SCOPE.into()
}

fn default_presence_text() -> String {
    DEFAULT_PRESENCE_TEXT.into()
}

fn default_enable_dashboard() -> bool {
    false
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

fn default_callback_url() -> String {
    DEFAULT_CALLBACK_URL.into()
}

/// Validate configuration values.
pub fn validate(cfg: &Config) -> anyhow::Result<()> {
    // Validate bot token is not empty
    anyhow::ensure!(
        !cfg.bot.token.trim().is_empty(),
        "config.bot.token must not be empty"
    );

    // Validate client_id is not empty and is a valid u64
    anyhow::ensure!(
        !cfg.bot.client_id.trim().is_empty(),
        "config.bot.client_id must not be empty"
    );
    cfg.bot.client_id.parse::<u64>().with_context(|| {
        format!(
            "config.bot.client_id '{}' must be a valid Discord snowflake (numeric)",
            cfg.bot.client_id
        )
    })?;

    // Validate guild_id if provided
    if !cfg.bot.guild_id.is_empty() {
        cfg.bot.guild_id.parse::<u64>().with_context(|| {
            format!(
                "config.bot.guild_id '{}' must be a valid Discord snowflake (numeric)",
                cfg.bot.guild_id
            )
        })?;
    }

    // Validate presence_type is in range 0-4
    anyhow::ensure!(
        cfg.bot.presence_type <= 4,
        "config.bot.presence_type must be 0-4 (Playing, Streaming, Listening, Watching, Competing), got {}",
        cfg.bot.presence_type
    );

    // Validate command_scope is either "guild" or "global"
    anyhow::ensure!(
        cfg.bot.command_scope == "guild" || cfg.bot.command_scope == "global",
        "config.bot.command_scope must be 'guild' or 'global', got '{}'",
        cfg.bot.command_scope
    );

    // Validate dashboard port is not zero
    anyhow::ensure!(
        cfg.dashboard.port > 0,
        "config.dashboard.port must be greater than 0, got {}",
        cfg.dashboard.port
    );

    // Validate MongoDB URI format if provided
    if !cfg.database.mongo_uri.is_empty() {
        anyhow::ensure!(
            cfg.database.mongo_uri.starts_with("mongodb://") || cfg.database.mongo_uri.starts_with("mongodb+srv://"),
            "config.database.mongo_uri must start with 'mongodb://' or 'mongodb+srv://', got '{}'",
            cfg.database.mongo_uri
        );
    }

    // Validate admin_ids are all valid u64s
    for admin_id in &cfg.dashboard.admin_ids {
        admin_id.parse::<u64>().with_context(|| {
            format!(
                "config.dashboard.admin_ids entry '{}' must be a valid Discord snowflake (numeric)",
                admin_id
            )
        })?;
    }

    Ok(())
}

/// Returns `true` when the bot token has not been set yet (first-run / setup mode).
/// Use this after `load()` to decide whether to start the setup wizard instead of the bot.
pub fn needs_setup(cfg: &Config) -> bool {
    cfg.bot.token.trim().is_empty()
}

/// Load `config.toml` from the current working directory.
///
/// If the file is absent, a default unconfigured `Config` is returned so that
/// `main` can detect setup mode via [`needs_setup`] and launch the setup wizard.
/// Validation is intentionally **not** performed here; call [`validate`] separately
/// once you have confirmed the configuration is complete.
pub fn load() -> anyhow::Result<Config> {
    let path = Path::new("config.toml");
    if !path.exists() {
        return Ok(Config {
            bot: BotConfig {
                token: String::new(),
                client_id: String::new(),
                guild_id: String::new(),
                command_scope: DEFAULT_COMMAND_SCOPE.to_string(),
                presence_text: DEFAULT_PRESENCE_TEXT.to_string(),
                presence_type: 0,
            },
            database: DatabaseConfig::default(),
            dashboard: DashboardConfig::default(),
        });
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file at '{}'", path.display()))?;
    toml::from_str(&raw).context("Failed to parse config.toml")
}

/// Serialize `Config` back to `config.toml` in the current working directory.
/// Validates the configuration before saving.
pub fn save(cfg: &Config) -> anyhow::Result<()> {
    validate(cfg)?;
    let raw = toml::to_string_pretty(cfg).context("Failed to serialize config")?;
    std::fs::write("config.toml", raw).context("Failed to write config.toml")?;
    Ok(())
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
        // No [dashboard] section → Default impl → enable_dashboard = false
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

    /// `DashboardConfig::default()` and the serde field default for `enable_dashboard`
    /// must agree so that the behaviour is the same whether the `[dashboard]` section
    /// is absent or present with all fields omitted.
    #[test]
    fn dashboard_enable_dashboard_defaults_are_consistent() {
        // Case 1: no [dashboard] section at all → uses DashboardConfig::default()
        let no_section: Config = toml::from_str(minimal_toml()).unwrap();
        assert!(!no_section.dashboard.enable_dashboard);

        // Case 2: [dashboard] section present but enable_dashboard omitted → uses serde default fn
        let with_section: Config = toml::from_str(
            r#"
[bot]
token = "t"
client_id = "c"

[dashboard]
port = 8080
"#,
        )
        .unwrap();
        assert!(!with_section.dashboard.enable_dashboard,
            "enable_dashboard serde default must match Default impl (both false)");

        // DashboardConfig::default() itself
        assert!(!DashboardConfig::default().enable_dashboard);
        assert_eq!(DashboardConfig::default().port, 8080);
        assert_eq!(
            DashboardConfig::default().callback_url,
            "http://localhost:8080/auth/discord/callback",
        );
    }

    #[test]
    fn config_round_trips_through_toml() {
        let original: Config = toml::from_str(
            r#"
[bot]
token = "tok"
client_id = "cid"
guild_id = "gid"
command_scope = "guild"
presence_text = "Playing"
presence_type = 3

[database]
mongo_uri = "mongodb://localhost"

[dashboard]
enable_dashboard = true
port = 9000
admin_ids = ["42"]
"#,
        )
        .unwrap();
        let serialized = toml::to_string_pretty(&original).unwrap();
        let restored: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(restored.bot.token, original.bot.token);
        assert_eq!(restored.bot.presence_type, original.bot.presence_type);
        assert_eq!(restored.database.mongo_uri, original.database.mongo_uri);
        assert_eq!(restored.dashboard.enable_dashboard, original.dashboard.enable_dashboard);
        assert_eq!(restored.dashboard.port, original.dashboard.port);
        assert_eq!(restored.dashboard.admin_ids, original.dashboard.admin_ids);
    }

    #[test]
    fn needs_setup_true_when_token_empty() {
        let cfg: Config = toml::from_str(
            r#"[bot]
token = ""
client_id = "123"
"#,
        )
        .unwrap();
        assert!(needs_setup(&cfg));
    }

    #[test]
    fn needs_setup_true_when_token_whitespace_only() {
        let cfg: Config = toml::from_str(
            r#"[bot]
token = "   "
client_id = "123"
"#,
        )
        .unwrap();
        assert!(needs_setup(&cfg));
    }

    #[test]
    fn needs_setup_false_when_token_set() {
        let cfg: Config = toml::from_str(minimal_toml()).unwrap();
        assert!(!needs_setup(&cfg));
    }

    #[test]
    fn load_returns_default_when_config_file_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = load();

        std::env::set_current_dir(&original_dir).unwrap();

        let cfg = result.expect("load() must succeed even when config.toml is absent");
        assert!(needs_setup(&cfg), "a missing config file should trigger setup mode");
        assert_eq!(cfg.dashboard.port, DEFAULT_PORT);
    }
}
