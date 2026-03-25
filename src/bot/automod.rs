use std::time::Duration;

use poise::serenity_prelude as serenity;
use tokio::time::timeout;

use crate::db::models::Warning;
use crate::state::SharedState;

const INVITE_PATTERNS: &[&str] = &["discord.gg/", "discord.com/invite/"];

/// Handle a new message for auto-moderation.
/// Checks for banned words, invite links, and spam.
pub async fn handle_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    state: SharedState,
) -> anyhow::Result<()> {
    // Ignore if automod is disabled
    let config = &state.config.automod;
    if !config.enabled {
        return Ok(());
    }

    // Ignore bots
    if message.author.bot {
        return Ok(());
    }

    // Ignore DMs (we need a guild for moderation)
    let guild_id = match message.guild_id {
        Some(id) => id,
        None => return Ok(()),
    };

    // Get the member to check permissions
    let member = match guild_id.member(ctx, message.author.id).await {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };

    // Ignore users with MANAGE_MESSAGES permission (moderators)
    #[allow(deprecated)]
    if member
        .permissions(ctx)
        .map(|p| p.contains(serenity::Permissions::MANAGE_MESSAGES))
        .unwrap_or(false)
    {
        return Ok(());
    }

    let content_lower = message.content.to_lowercase();

    // Check for banned words (keyword filter)
    if !config.banned_words.is_empty() {
        for word in &config.banned_words {
            let word_lower = word.to_lowercase();
            if content_lower.contains(&word_lower) {
                handle_violation(
                    ctx,
                    message,
                    &state,
                    "Banned word detected",
                    "Message contained a banned word",
                )
                .await?;
                return Ok(());
            }
        }
    }

    // Check for invite links (invite blocker)
    if config.invite_blocker_enabled {
        for pattern in INVITE_PATTERNS {
            if content_lower.contains(pattern) {
                handle_violation(
                    ctx,
                    message,
                    &state,
                    "Invite link detected",
                    "Message contained a Discord invite link",
                )
                .await?;
                return Ok(());
            }
        }
    }

    // Check for spam (anti-spam)
    if config.anti_spam_enabled
        && check_spam(
            &state,
            guild_id,
            message.author.id,
            config.spam_threshold,
            config.spam_interval_secs,
        )
        .await?
    {
        handle_violation(
            ctx,
            message,
            &state,
            "Spam detected",
            "User exceeded message rate limit",
        )
        .await?;
        return Ok(());
    }

    Ok(())
}

/// Check if a user is spamming based on message rate.
async fn check_spam(
    state: &SharedState,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    threshold: u8,
    interval_secs: u64,
) -> anyhow::Result<bool> {
    let mut message_counts = state.message_counts.lock().await;
    let key = (guild_id.get(), user_id.get());
    let now = std::time::Instant::now();
    let interval = Duration::from_secs(interval_secs);

    let (is_spam, new_count) = if let Some((count, start)) = message_counts.get(&key).cloned() {
        let elapsed = now.duration_since(start);
        if elapsed > interval {
            // Reset the window
            (false, 1)
        } else if count >= threshold {
            // User has exceeded the threshold within the interval
            // Reset for next time
            (true, 1)
        } else {
            // Increment count
            (false, count + 1)
        }
    } else {
        // First message in window
        (false, 1)
    };

    message_counts.insert(key, (new_count, now));
    Ok(is_spam)
}

/// Handle a violation: delete the message and log a warning.
async fn handle_violation(
    ctx: &serenity::Context,
    message: &serenity::Message,
    state: &SharedState,
    violation_type: &str,
    reason: &str,
) -> anyhow::Result<()> {
    // Delete the message
    let delete_result = timeout(Duration::from_secs(5), message.delete(ctx)).await;
    if delete_result.is_err() {
        tracing::warn!("Failed to delete automod message");
    }

    // Log warning to database if available
    if let Some(db) = state.database() {
        let guild_id = message.guild_id.map(|g| g.get().to_string()).unwrap_or_default();
        let user_id = message.author.id.get().to_string();
        let bot_user_id = ctx.cache.current_user().id.get().to_string();

        if let Err(e) = Warning::create(&db, &guild_id, &user_id, &bot_user_id, reason).await {
            tracing::warn!("Failed to log automod warning to database: {}", e);
        }
    }

    tracing::info!(
        "Auto-mod: {} - {} by user {} in guild {}",
        violation_type,
        reason,
        message.author.id,
        message.guild_id.map(|g| g.get()).unwrap_or(0)
    );

    Ok(())
}
