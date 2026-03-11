pub mod commands;

use std::sync::Arc;

use poise::serenity_prelude as serenity;
use tracing::info;

use crate::state::SharedState;

/// Poise error type used across all commands.
pub type Error = anyhow::Error;

/// Poise context type.  `SharedState` is our `Data` type parameter.
pub type Context<'a> = poise::Context<'a, SharedState, Error>;

/// Start the Discord bot and block until it shuts down.
pub async fn start(state: SharedState) -> anyhow::Result<()> {
    let token = state.config.bot.token.clone();
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::MESSAGE_CONTENT;

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
                        let guild_id = serenity::GuildId::new(
                            cfg.guild_id
                                .parse::<u64>()
                                .expect("config.bot.guild_id must be a valid u64"),
                        );
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

                    // Set initial bot presence
                    let activity = presence_activity(cfg.presence_type, &cfg.presence_text);
                    ctx.set_presence(Some(activity), serenity::OnlineStatus::Online);

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
}
