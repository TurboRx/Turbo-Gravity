# Turbo Gravity

A powerful, feature-rich Discord bot with an **optional web-based Control Panel**. Rewritten in **Rust** using [Poise](https://github.com/serenity-rs/poise) + [Serenity](https://github.com/serenity-rs/serenity) for the bot and [Axum](https://github.com/tokio-rs/axum) for the dashboard API, with [Tokio](https://tokio.rs) as the async runtime.

## Architecture

```
main.rs
├── Arc<AppState>          ← shared state (config + optional MongoDB pool)
│
├── tokio::spawn → dashboard::serve()   ← optional Axum REST API (port 8080)
│                  State<Arc<AppState>> ← same Arc injected into every route
│
└── bot::start()           ← Poise framework (blocks until shutdown)
       ctx.data()          ← SharedState accessible in every command
```

### Key design decisions

| Concern | Solution |
|---|---|
| Command framework | `poise 0.6` with `#[poise::command(slash_command)]` |
| Shared state | `Arc<AppState>` — passed as Poise `Data` type and Axum `State` |
| Dashboard toggle | `enable_dashboard = true/false` in `config.toml` |
| Dashboard API | `axum 0.8` spawned via `tokio::spawn` before bot blocks |
| Database | `mongodb 3` driver; bot operates without DB if URI is empty |
| Configuration | `config.toml` (TOML) loaded at startup; `.env` also supported |
| Async runtime | `tokio` with `full` features |

---

## Features

### Bot Commands

| Category | Commands |
|---|---|
| **Fun** | `/daily`, `/work`, `/balance`, `/coinflip`, `/roll`, `/8ball` |
| **Misc** | `/choose`, `/poll`, `/remind` |
| **Moderation** | `/ban`, `/kick`, `/timeout`, `/warn`, `/warnings`, `/purge`, `/slowmode`, `/lock`, `/unlock`, `/unban` |
| **Tickets** | `/ticket create`, `/ticket close`, `/ticket add`, `/ticket remove` |
| **Utility** | `/ping`, `/uptime`, `/stats`, `/help`, `/userinfo`, `/serverinfo`, `/channelinfo`, `/roleinfo`, `/avatar`, `/embed`, `/contime` |

### Optional Dashboard API

When `enable_dashboard = true` in `config.toml`, an Axum HTTP server starts in parallel with the bot:

| Route | Description |
|---|---|
| `GET /health` | Liveness probe — returns `{"status":"ok","version":"..."}` |
| `GET /api/stats` | Runtime stats (DB connected, bot configured, port) |
| `GET /api/config` | Public (non-secret) config values |

---

## Quick Start

### Prerequisites
- Rust (stable, 1.75+) — install via [rustup](https://rustup.rs)
- MongoDB (optional — bot works without a database)
- A Discord bot token from the [Discord Developer Portal](https://discord.com/developers/applications)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/TurboRx/Turbo-Gravity.git
   cd Turbo-Gravity
   ```

2. **Configure the bot**
   ```bash
   # config.toml is already present — edit it with your values
   nano config.toml
   ```
   At minimum you need `bot.token`. Everything else has sensible defaults.

3. **Build and run**
   ```bash
   cargo run --release
   ```

The bot registers slash commands to the guild specified by `guild_id` (instant) or globally (up to 1 hour delay) based on `command_scope`.

---

## Configuration (`config.toml`)

```toml
[bot]
token        = ""              # Required: Discord bot token
client_id    = ""              # Discord application/client ID
guild_id     = ""              # Leave empty for global command registration
command_scope = "guild"        # "guild" (instant) or "global"
presence_text = "Ready to serve"
presence_type = 0              # 0=Playing 1=Streaming 2=Listening 3=Watching 4=Competing

[database]
mongo_uri = ""                 # Optional: MongoDB URI; leave empty to run without DB

[dashboard]
enable_dashboard = true        # Set to false to disable the web API entirely
port             = 8080
session_secret   = ""          # Reserved for future OAuth2 session signing
client_secret    = ""          # Reserved for future Discord OAuth2 login
callback_url     = "http://localhost:8080/auth/discord/callback"
admin_ids        = []          # Reserved for future admin access control
```

Environment variables in a `.env` file are loaded automatically (via `dotenvy`) and can be used alongside `config.toml`.

---

## Docker Deployment

### Build locally

```bash
docker build -t turbo-gravity .
```

### Run

```bash
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  --name turbo-gravity \
  turbo-gravity
```

The binary reads `config.toml` from its working directory at startup. Mount your config at `/app/config.toml` — **never bake secrets into the image**.

---

## Development

### Adding a new command

1. Create `src/bot/commands/<category>/mycommand.rs`
2. Write the handler following the Poise pattern:
   ```rust
   use crate::bot::{Context, Error};

   /// Brief description shown in /help
   #[poise::command(slash_command, ephemeral)]
   pub async fn mycommand(ctx: Context<'_>) -> Result<(), Error> {
       ctx.say("Hello!").await?;
       Ok(())
   }
   ```
3. Add `mod mycommand;` and include `mycommand::mycommand()` in the `commands()` vec in `src/bot/commands/<category>/mod.rs`

### Accessing shared state inside a command

```rust
// ctx.data() returns &Arc<AppState>
let db = match ctx.data().database() {
    Some(db) => db,
    None => { ctx.say("No database configured.").await?; return Ok(()); }
};
```

### Lint / check

```bash
cargo check          # fast type-check
cargo clippy         # lints
cargo build --release
```

---

## Security Notes

- **Never commit `config.toml`** with real tokens — it is listed in `.gitignore`
- `session_secret` and `client_secret` are reserved for future OAuth2 support
- Use HTTPS in production via a reverse proxy (nginx, Caddy, Traefik)
- MongoDB URI should use credentials in production
- The dashboard API has **no authentication** in the current release — restrict access at the network/proxy level until OAuth2 is implemented

---

## License

[MIT](LICENSE)

---

## Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for commit guidelines and the PR process. For major changes, open an issue first.

---

## Support

For issues or questions, open an issue on [GitHub](https://github.com/TurboRx/Turbo-Gravity/issues).
