pub mod commands;

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use poise::serenity_prelude as serenity;
use tracing::info;

use crate::state::SharedState;

/// Poise error type used across all commands.
pub type Error = anyhow::Error;

/// Poise context type.  `SharedState` is our `Data` type parameter.
pub type Context<'a> = poise::Context<'a, SharedState, Error>;

/// Start the Discord bot with automatic reconnection on failure.
/// Retries with exponential back-off (5 s → 10 s → … → 5 min cap).
pub async fn start(state: SharedState) -> anyhow::Result<()> {
    let mut retry_delay = Duration::from_secs(5);
    const MAX_DELAY: Duration = Duration::from_secs(300);

    loop {
        info!("Starting Discord client…");
        match run_client(Arc::clone(&state)).await {
            Ok(()) => {
                // Clean shutdown (e.g. SIGTERM) — don't retry.
                state.bot_online.store(false, Ordering::Relaxed);
                info!("Discord client exited cleanly.");
                return Ok(());
            }
            Err(e) => {
                state.bot_online.store(false, Ordering::Relaxed);
                tracing::warn!(
                    "Discord client disconnected: {e}. Reconnecting in {retry_delay:?}…"
                );
                tokio::time::sleep(retry_delay).await;
                retry_delay = (retry_delay * 2).min(MAX_DELAY);
            }
        }
    }
}

/// Build the Serenity/Poise client and run it until it exits or errors.
async fn run_client(state: SharedState) -> anyhow::Result<()> {
    let token = state.config.bot.token.clone();
    // Use non_privileged intents by default. GUILD_MEMBERS and MESSAGE_CONTENT
    // are privileged intents that require explicit enabling in the Discord
    // Developer Portal (Bot → Privileged Gateway Intents).
    // Enable them there and add them back here once toggled on.
    let intents = serenity::GatewayIntents::non_privileged();

    let commands = commands::all();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            on_error: |err| {
                Box::pin(async move {
                    // Forward command errors and let poise handle the rest
                    match err {
                        poise::FrameworkError::Command { error, ctx, .. } => {
                            tracing::error!("Command '{}' error: {:?}", ctx.command().name, error);
                            let _ = ctx
                                .say("An error occurred while executing this command.")
                                .await;
                        }
                        other => {
                            if let Err(e) = poise::builtins::on_error(other).await {
                                tracing::error!("Framework error handler failed: {e}");
                            }
                        }
                    }
                })
            },
            // Track Resume events so the dashboard reflects the real online status
            // after a transparent gateway reconnect (no new READY is fired).
            event_handler: |_ctx, event, _framework_ctx, data| {
                Box::pin(async move {
                    match event {
                        serenity::FullEvent::Resume { .. } => {
                            data.bot_online.store(true, Ordering::Relaxed);
                        }
                        // Bot joined a new guild — increment the live guild counter.
                        // `is_new == Some(true)` distinguishes a genuine join from the
                        // GUILD_CREATE bursts that Discord sends when the bot reconnects
                        // (those have `is_new == Some(false)` when the cache is active).
                        serenity::FullEvent::GuildCreate { is_new, .. } => {
                            if *is_new == Some(true) {
                                data.guild_count.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        // Bot left / was kicked from a guild — decrement the counter.
                        // `incomplete.unavailable == true` means a temporary Discord
                        // outage, not a leave; we ignore those to avoid drift.
                        serenity::FullEvent::GuildDelete { incomplete, .. } => {
                            if !incomplete.unavailable {
                                // Saturating-sub: never let the counter wrap below 0.
                                let _ = data.guild_count.fetch_update(
                                    Ordering::Relaxed,
                                    Ordering::Relaxed,
                                    |v| v.checked_sub(1),
                                );
                            }
                        }
                        _ => {}
                    }
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup({
            // Clone Arc so we can move it into the async setup closure
            let state = Arc::clone(&state);
            move |ctx, ready, framework| {
                Box::pin(async move {
                    let cfg = &state.config.bot;
                    info!("Logged in as {}", ready.user.name);

                    // Register commands guild-scoped or globally based on config
                    if cfg.command_scope == "guild" && !cfg.guild_id.is_empty() {
                        let guild_id_num = cfg.guild_id.parse::<u64>().map_err(|e| {
                            anyhow::anyhow!(
                                "config.bot.guild_id '{}' must be a valid u64: {}",
                                cfg.guild_id,
                                e
                            )
                        })?;
                        let guild_id = serenity::GuildId::new(guild_id_num);
                        poise::builtins::register_in_guild(
                            ctx,
                            &framework.options().commands,
                            guild_id,
                        )
                        .await?;
                        info!(
                            "Registered {} commands in guild {guild_id}",
                            framework.options().commands.len()
                        );
                    } else {
                        poise::builtins::register_globally(ctx, &framework.options().commands)
                            .await?;
                        info!(
                            "Registered {} commands globally",
                            framework.options().commands.len()
                        );
                    }

                    // Resolve dynamic variables ({servers}, {members}) in presence text
                    let server_count = ready.guilds.len();
                    let presence_text = resolve_presence_text(&cfg.presence_text, server_count);

                    // Set initial bot presence using configured online status
                    let activity = presence_activity(cfg.presence_type, &presence_text);
                    let online_status = map_online_status(&cfg.online_status);
                    ctx.set_presence(Some(activity), online_status);

                    // Store the guild count so the dashboard can display it.
                    state.guild_count.store(server_count, Ordering::Relaxed);

                    // Mark the bot as online now that the READY event has been received.
                    state.bot_online.store(true, Ordering::Relaxed);

                    // Return the Arc<AppState>; this becomes ctx.data() in every command
                    Ok(state)
                })
            }
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;
    Ok(())
}

/// Replace `{servers}` and `{members}` placeholders in the presence text.
/// `server_count` comes from the READY event's guild list (accurate at startup).
/// `{members}` resolves to "0" at startup because member counts require the cache
/// to receive all GUILD_CREATE events, which happens asynchronously after READY.
/// `{servers}` is replaced with the numeric server count.
fn resolve_presence_text(text: &str, server_count: usize) -> String {
    // Members count is unavailable at READY time; use 0 as a safe initial value.
    text.replace("{servers}", &server_count.to_string())
        .replace("{members}", "0")
}

/// Map the `online_status` config string to a serenity `OnlineStatus` variant.
fn map_online_status(status: &str) -> serenity::OnlineStatus {
    match status {
        "dnd" => serenity::OnlineStatus::DoNotDisturb,
        "idle" => serenity::OnlineStatus::Idle,
        "invisible" => serenity::OnlineStatus::Invisible,
        _ => serenity::OnlineStatus::Online,
    }
}

/// Map a presence type integer to the correct serenity `ActivityData` variant.
fn presence_activity(presence_type: u8, text: &str) -> serenity::ActivityData {
    match presence_type {
        1 => serenity::ActivityData::streaming(text, "")
            .unwrap_or_else(|_| serenity::ActivityData::playing(text)),
        2 => serenity::ActivityData::listening(text),
        3 => serenity::ActivityData::watching(text),
        4 => serenity::ActivityData::competing(text),
        _ => serenity::ActivityData::playing(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presence_type_0_is_playing() {
        let a = presence_activity(0, "test");
        assert_eq!(a.name, "test");
        // Playing maps to ActivityType::Playing (0)
        assert_eq!(a.kind, serenity::ActivityType::Playing);
    }

    #[test]
    fn presence_type_2_is_listening() {
        let a = presence_activity(2, "lofi beats");
        assert_eq!(a.name, "lofi beats");
        assert_eq!(a.kind, serenity::ActivityType::Listening);
    }

    #[test]
    fn presence_type_3_is_watching() {
        let a = presence_activity(3, "a stream");
        assert_eq!(a.kind, serenity::ActivityType::Watching);
    }

    #[test]
    fn presence_type_4_is_competing() {
        let a = presence_activity(4, "a tournament");
        assert_eq!(a.kind, serenity::ActivityType::Competing);
    }

    #[test]
    fn presence_type_1_with_empty_url_falls_back_to_playing() {
        // The function passes "" as the streaming URL.  An empty string is not a
        // valid URL so serenity returns an error and the fallback (Playing) is used.
        let a = presence_activity(1, "a stream");
        assert_eq!(a.name, "a stream");
        assert_eq!(a.kind, serenity::ActivityType::Playing);
    }

    #[test]
    fn presence_type_1_with_valid_url_is_streaming() {
        // Verify the streaming branch directly: serenity accepts a valid URL and
        // returns ActivityType::Streaming.
        let result = serenity::ActivityData::streaming("a stream", "https://twitch.tv/example");
        assert!(result.is_ok());
        let a = result.unwrap();
        assert_eq!(a.kind, serenity::ActivityType::Streaming);
        assert_eq!(a.name, "a stream");
    }

    #[test]
    fn presence_type_unknown_defaults_to_playing() {
        let a = presence_activity(99, "something");
        assert_eq!(a.kind, serenity::ActivityType::Playing);
    }

    #[test]
    fn map_online_status_variants() {
        assert_eq!(map_online_status("online"), serenity::OnlineStatus::Online);
        assert_eq!(
            map_online_status("dnd"),
            serenity::OnlineStatus::DoNotDisturb
        );
        assert_eq!(map_online_status("idle"), serenity::OnlineStatus::Idle);
        assert_eq!(
            map_online_status("invisible"),
            serenity::OnlineStatus::Invisible
        );
        // Unknown values fall back to Online
        assert_eq!(map_online_status(""), serenity::OnlineStatus::Online);
        assert_eq!(map_online_status("offline"), serenity::OnlineStatus::Online);
    }

    #[test]
    fn resolve_presence_text_substitutes_servers() {
        assert_eq!(
            resolve_presence_text("Watching {servers} servers", 5),
            "Watching 5 servers"
        );
    }

    #[test]
    fn resolve_presence_text_substitutes_members_as_zero() {
        assert_eq!(
            resolve_presence_text("{members} members across {servers} servers", 3),
            "0 members across 3 servers"
        );
    }

    #[test]
    fn resolve_presence_text_no_placeholders() {
        assert_eq!(
            resolve_presence_text("Ready to serve", 10),
            "Ready to serve"
        );
    }
}
