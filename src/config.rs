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
    #[serde(default)]
    pub automod: AutoModConfig,
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
    /// Discord online status: "online" | "dnd" | "idle" | "invisible"
    #[serde(default = "default_online_status")]
    pub online_status: String,
    /// Optional URL to a bot avatar image (fetched and applied at startup).
    #[serde(default)]
    pub avatar_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub mongo_uri: String,
}

/// Dashboard / `OAuth2` configuration loaded from `[dashboard]` section.
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

/// Auto-moderation configuration loaded from `[automod]` section of `config.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AutoModConfig {
    #[serde(default = "default_automod_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub banned_words: Vec<String>,
    #[serde(default = "default_invite_blocker_enabled")]
    pub invite_blocker_enabled: bool,
    #[serde(default = "default_anti_spam_enabled")]
    pub anti_spam_enabled: bool,
    #[serde(default = "default_spam_threshold")]
    pub spam_threshold: u8,
    #[serde(default = "default_spam_interval")]
    pub spam_interval_secs: u64,
}

impl Default for AutoModConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            banned_words: Vec::new(),
            invite_blocker_enabled: false,
            anti_spam_enabled: false,
            spam_threshold: DEFAULT_SPAM_THRESHOLD,
            spam_interval_secs: DEFAULT_SPAM_INTERVAL_SECS,
        }
    }
}

pub const DEFAULT_SPAM_THRESHOLD: u8 = 5;
pub const DEFAULT_SPAM_INTERVAL_SECS: u64 = 10;

const fn default_automod_enabled() -> bool {
    false
}

const fn default_invite_blocker_enabled() -> bool {
    false
}

const fn default_anti_spam_enabled() -> bool {
    false
}

const fn default_spam_threshold() -> u8 {
    DEFAULT_SPAM_THRESHOLD
}

const fn default_spam_interval() -> u64 {
    DEFAULT_SPAM_INTERVAL_SECS
}

pub const DEFAULT_COMMAND_SCOPE: &str = "guild";
pub const DEFAULT_PRESENCE_TEXT: &str = "Ready to serve";
pub const DEFAULT_CALLBACK_URL: &str = "http://localhost:8080/auth/callback";
pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_ONLINE_STATUS: &str = "online";

fn default_command_scope() -> String {
    DEFAULT_COMMAND_SCOPE.into()
}

fn default_presence_text() -> String {
    DEFAULT_PRESENCE_TEXT.into()
}

fn default_online_status() -> String {
    DEFAULT_ONLINE_STATUS.into()
}

const fn default_enable_dashboard() -> bool {
    false
}

const fn default_port() -> u16 {
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

    // Validate online_status
    anyhow::ensure!(
        matches!(
            cfg.bot.online_status.as_str(),
            "online" | "dnd" | "idle" | "invisible"
        ),
        "config.bot.online_status must be 'online', 'dnd', 'idle', or 'invisible', got '{}'",
        cfg.bot.online_status
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
            cfg.database.mongo_uri.starts_with("mongodb://")
                || cfg.database.mongo_uri.starts_with("mongodb+srv://"),
            "config.database.mongo_uri must start with 'mongodb://' or 'mongodb+srv://', got '{}'",
            cfg.database.mongo_uri
        );
    }

    // Validate admin_ids are all valid u64s
    for admin_id in &cfg.dashboard.admin_ids {
        admin_id.parse::<u64>().with_context(|| {
            format!(
                "config.dashboard.admin_ids entry '{admin_id}' must be a valid Discord snowflake (numeric)"
            )
        })?;
    }

    // Validate auto-mod spam_threshold is at least 1
    anyhow::ensure!(
        cfg.automod.spam_threshold >= 1,
        "config.automod.spam_threshold must be at least 1, got {}",
        cfg.automod.spam_threshold
    );

    // Validate auto-mod spam_interval_secs is at least 1
    anyhow::ensure!(
        cfg.automod.spam_interval_secs >= 1,
        "config.automod.spam_interval_secs must be at least 1, got {}",
        cfg.automod.spam_interval_secs
    );

    Ok(())
}

/// Returns `true` when the configuration is not yet sufficient to run the bot
/// (first-run / setup mode). Currently this requires both a non-empty token and
/// a non-empty, numeric `client_id`.
/// Use this after `load()` to decide whether to start the setup wizard instead of the bot.
pub fn needs_setup(cfg: &Config) -> bool {
    let token_empty = cfg.bot.token.trim().is_empty();
    let client_id_str = cfg.bot.client_id.trim();

    let client_id_invalid = client_id_str.is_empty() || client_id_str.parse::<u64>().is_err();

    token_empty || client_id_invalid
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
        let token = std::env::var("DISCORD_TOKEN").unwrap_or_default();
        let client_id = std::env::var("DISCORD_APPLICATION_ID").unwrap_or_default();
        let client_secret = std::env::var("DISCORD_CLIENT_SECRET").unwrap_or_default();
        let admin_id = std::env::var("ADMIN_DISCORD_ID").unwrap_or_default();
        let admin_ids = if admin_id.is_empty() {
            vec![]
        } else {
            vec![admin_id]
        };
        return Ok(Config {
            bot: BotConfig {
                token,
                client_id,
                guild_id: String::new(),
                command_scope: DEFAULT_COMMAND_SCOPE.to_string(),
                presence_text: DEFAULT_PRESENCE_TEXT.to_string(),
                presence_type: 0,
                online_status: DEFAULT_ONLINE_STATUS.to_string(),
                avatar_url: String::new(),
            },
            database: DatabaseConfig::default(),
            dashboard: DashboardConfig {
                enable_dashboard: true,
                port: DEFAULT_PORT,
                session_secret: String::new(),
                client_secret,
                callback_url: DEFAULT_CALLBACK_URL.into(),
                admin_ids,
            },
            automod: AutoModConfig::default(),
        });
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file at '{}'", path.display()))?;
    let mut cfg: Config = toml::from_str(&raw).context("Failed to parse config.toml")?;

    // Allow environment variables to override sensitive config values so that
    // tokens and secrets never have to be stored in config.toml on disk.
    if let Ok(token) = std::env::var("DISCORD_TOKEN") {
        if !token.trim().is_empty() {
            cfg.bot.token = token;
        }
    }
    if let Ok(secret) = std::env::var("DISCORD_CLIENT_SECRET") {
        if !secret.trim().is_empty() {
            cfg.dashboard.client_secret = secret;
        }
    }
    if let Ok(admin_id) = std::env::var("ADMIN_DISCORD_ID") {
        if !admin_id.trim().is_empty() && cfg.dashboard.admin_ids.is_empty() {
            cfg.dashboard.admin_ids = vec![admin_id];
        }
    }
    if let Ok(client_id) = std::env::var("DISCORD_APPLICATION_ID") {
        if !client_id.trim().is_empty() && cfg.bot.client_id.trim().is_empty() {
            cfg.bot.client_id = client_id;
        }
    }

    Ok(cfg)
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
        assert_eq!(cfg.bot.online_status, "online");
        assert!(cfg.bot.avatar_url.is_empty());
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
        assert!(
            !with_section.dashboard.enable_dashboard,
            "enable_dashboard serde default must match Default impl (both false)"
        );

        // DashboardConfig::default() itself
        assert!(!DashboardConfig::default().enable_dashboard);
        assert_eq!(DashboardConfig::default().port, 8080);
        assert_eq!(
            DashboardConfig::default().callback_url,
            "http://localhost:8080/auth/callback",
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
        assert_eq!(
            restored.dashboard.enable_dashboard,
            original.dashboard.enable_dashboard
        );
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
        assert!(
            needs_setup(&cfg),
            "a missing config file should trigger setup mode"
        );
        assert_eq!(cfg.dashboard.port, DEFAULT_PORT);
        assert_eq!(cfg.bot.online_status, DEFAULT_ONLINE_STATUS);
    }

    #[test]
    fn validate_rejects_invalid_online_status() {
        let mut cfg: Config = toml::from_str(minimal_toml()).unwrap();
        cfg.bot.online_status = "away".to_string(); // not a valid value
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn validate_accepts_all_valid_online_statuses() {
        let mut cfg: Config = toml::from_str(minimal_toml()).unwrap();
        for status in &["online", "dnd", "idle", "invisible"] {
            cfg.bot.online_status = (*status).to_string();
            assert!(validate(&cfg).is_ok(), "expected ok for status={status}");
        }
    }
}
