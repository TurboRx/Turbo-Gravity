# Contributing to Turbo Gravity

Thank you for your interest in contributing to Turbo Gravity! This guide covers the standard Rust workflow for getting started, submitting changes, and keeping code quality high.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Style Guidelines](#code-style-guidelines)
- [Commit Message Guidelines](#commit-message-guidelines)
- [Pull Request Process](#pull-request-process)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Enhancements](#suggesting-enhancements)

## Code of Conduct

This project and everyone participating in it is expected to uphold a respectful and welcoming environment. Please be kind and courteous to others.

## Getting Started

### Prerequisites

- **Rust (stable)** — install via [rustup](https://rustup.rs):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Git**
- **MongoDB** (optional — the bot runs without a database)
- A Discord bot token from the [Discord Developer Portal](https://discord.com/developers/applications)

### Setting Up Your Development Environment

1. **Fork and Clone**
   ```bash
   git clone https://github.com/YOUR_USERNAME/Turbo-Gravity.git
   cd Turbo-Gravity
   ```

2. **Configure the bot**
   ```bash
   # Edit config.toml with your Discord bot token at minimum
   nano config.toml
   ```

3. **Check the build**
   ```bash
   cargo check
   ```

4. **Run in development mode**
   ```bash
   RUST_LOG=debug cargo run
   ```

### Project Structure

```
Turbo-Gravity/
├── src/
│   ├── bot/
│   │   ├── commands/        # Slash command handlers (grouped by category)
│   │   │   ├── fun/         # Economy commands (daily, work, balance, …)
│   │   │   ├── misc/        # Misc commands (poll, remind, choose)
│   │   │   ├── moderation/  # Mod tools (ban, kick, warn, …)
│   │   │   ├── tickets/     # Ticket system
│   │   │   └── utility/     # Utility commands (ping, userinfo, …)
│   │   └── mod.rs           # Poise framework setup
│   ├── dashboard/           # Optional Axum HTTP dashboard
│   ├── db/                  # MongoDB connection + data models
│   ├── config.rs            # Config deserialization (config.toml)
│   ├── state.rs             # Shared AppState (Arc<AppState>)
│   └── main.rs              # Entry point, graceful shutdown
├── config.toml              # Bot configuration (not committed with secrets)
├── Cargo.toml               # Rust dependencies
├── Dockerfile               # Multi-stage Docker build
└── rust-toolchain.toml      # Pins the Rust toolchain channel
```

## Development Workflow

### Branching Strategy

- `main` — Production-ready code
- `feature/*` — New features
- `bugfix/*` — Bug fixes
- `docs/*` — Documentation changes

### Making Changes

1. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** in small, logical commits

3. **Run the standard checks** (same checks run in CI):
   ```bash
   cargo check                    # fast type-check
   cargo clippy -- -D warnings    # lints — must pass with zero warnings
   cargo build --release          # verify release build
   ```

4. **Keep your branch updated** with main:
   ```bash
   git fetch origin
   git rebase origin/main
   ```

## Code Style Guidelines

### Rust

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting — run `cargo fmt` before committing
- Pass `cargo clippy -- -D warnings` with zero errors before submitting a PR
- Use modern idioms: `let-else` for early-return patterns, `?` for error propagation
- Prefer `anyhow::Result` for command-level errors and `thiserror` for library-level types
- Use `tracing::info!` / `tracing::error!` for logging — **never** `println!` in production paths

### Adding a New Command

1. Create `src/bot/commands/<category>/mycommand.rs`
2. Write the handler:
   ```rust
   use crate::bot::{Context, Error};

   /// Brief description shown in Discord's /help autocomplete
   #[poise::command(slash_command, ephemeral)]
   pub async fn mycommand(ctx: Context<'_>) -> Result<(), Error> {
       ctx.say("Hello!").await?;
       Ok(())
   }
   ```
3. Register it in `src/bot/commands/<category>/mod.rs`:
   ```rust
   mod mycommand;

   pub fn commands() -> Vec<Command<SharedState, Error>> {
       vec![
           // … existing commands …
           mycommand::mycommand(),
       ]
   }
   ```

### Accessing the Database

Use the `let-else` idiom for early-return on missing DB:

```rust
let Some(db) = ctx.data().database() else {
    ctx.say("Database is unavailable.").await?;
    return Ok(());
};
```

### Guild-only Commands

Add `guild_only` to the command attribute and guard against DM invocations:

```rust
#[poise::command(slash_command, guild_only, required_permissions = "MODERATE_MEMBERS")]
pub async fn mymod(ctx: Context<'_>) -> Result<(), Error> {
    // guild_only + poise guarantees ctx.guild_id() is Some
    let guild_id = ctx.guild_id().unwrap();
    // …
}
```

## Commit Message Guidelines

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to…" not "Moves cursor to…")
- Limit the first line to 72 characters or less
- Reference issues and pull requests when applicable

Examples:
```
Add /leaderboard command for top economy users
Fix infinite level-up loop in /daily XP calculation
Update README with Docker deployment instructions
Refactor DB upsert to use let-else idiom
```

## Pull Request Process

1. **Ensure all CI checks pass**:
   - `cargo check` ✅
   - `cargo clippy -- -D warnings` ✅
   - `cargo build --release` ✅

2. **Update documentation** if you changed or added functionality

3. **PR Template**:
   ```markdown
   ## Description
   Brief description of changes

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update

   ## Checklist
   - [ ] `cargo clippy -- -D warnings` passes
   - [ ] Code follows project style guidelines
   - [ ] Related issues are referenced

   ## Related Issues
   Fixes #(issue)
   ```

4. **Review Process**:
   - Maintainers will review your PR
   - Address any feedback or requested changes
   - Once approved, your PR will be merged

## Reporting Bugs

Found a bug? Please create an issue with:

- **Title**: Short, descriptive title
- **Description**: Detailed description of the issue
- **Steps to Reproduce**: Step-by-step instructions
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Environment**: OS, Rust version (`rustc --version`), etc.
- **Logs**: Set `RUST_LOG=debug` and include relevant output

## Suggesting Enhancements

Have an idea? Create an enhancement issue with:

- **Title**: Clear feature request title
- **Description**: Detailed description of the enhancement
- **Motivation**: Why this feature would be useful
- **Implementation Ideas**: Your thoughts on implementation (optional)

---

## Questions?

- Open an issue with the `question` label
- Join our community discussions on [GitHub Discussions](https://github.com/TurboRx/Turbo-Gravity/discussions)

## Recognition

Contributors will be recognized in our release notes and project documentation. Thank you for making Turbo Gravity better!

---

## License

By contributing to Turbo Gravity, you agree that your contributions will be licensed under the [MIT License](LICENSE).
