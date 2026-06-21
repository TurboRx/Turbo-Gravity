use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Temporarily ban a user; they are automatically unbanned after the duration
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS"
)]
pub async fn tempban(
    ctx: Context<'_>,
    #[description = "User to ban"] user: serenity::User,
    #[description = "Ban duration in minutes (1-43200 = up to 30 days)"]
    #[min = 1_u32]
    #[max = 43200_u32]
    duration: u32,
    #[description = "Days of messages to delete (0-7)"]
    #[min = 0_u8]
    #[max = 7_u8]
    delete_days: Option<u8>,
    #[description = "Reason for the ban"] reason: Option<String>,
) -> Result<(), Error> {
    // Extract guild data before any await
    let (guild_id, guild_name) = {
        let guild = ctx
            .guild()
            .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
        (guild.id, guild.name.clone())
    };

    if user.id == ctx.author().id {
        ctx.say("You cannot ban yourself.").await?;
        return Ok(());
    }
    if user.id == ctx.cache().current_user().id {
        ctx.say("I cannot ban myself.").await?;
        return Ok(());
    }

    let reason = reason
        .as_deref()
        .unwrap_or("No reason provided")
        .to_string();
    let delete_days = delete_days.unwrap_or(0);

    // Role hierarchy check
    if let Ok(target_member) = guild_id.member(ctx, user.id).await {
        let invoker = ctx.author_member().await;
        if let Some(invoker) = invoker {
            let invoker_top = invoker
                .roles(ctx)
                .map_or(0, |roles| roles.iter().map(|r| r.position).max().unwrap_or(0));
            let target_top = target_member
                .roles(ctx)
                .map_or(0, |roles| roles.iter().map(|r| r.position).max().unwrap_or(0));
            if invoker_top <= target_top {
                ctx.say("You cannot ban someone with an equal or higher role.")
                    .await?;
                return Ok(());
            }
        }

        // Bot hierarchy check
        if let Ok(bot_member) = guild_id.member(ctx, ctx.cache().current_user().id).await {
            let bot_top = bot_member
                .roles(ctx)
                .map_or(0, |roles| roles.iter().map(|r| r.position).max().unwrap_or(0));
            let target_top = target_member
                .roles(ctx)
                .map_or(0, |roles| roles.iter().map(|r| r.position).max().unwrap_or(0));
            if bot_top <= target_top {
                ctx.say("I cannot ban this user — their role is equal to or higher than mine.")
                    .await?;
                return Ok(());
            }
        }

        // DM the user before the ban
        let days_display = duration / (24 * 60);
        let hours_display = (duration % (24 * 60)) / 60;
        let mins_display = duration % 60;
        let duration_str = if days_display > 0 {
            format!("{days_display}d {hours_display}h {mins_display}m")
        } else if hours_display > 0 {
            format!("{hours_display}h {mins_display}m")
        } else {
            format!("{duration}m")
        };

        let dm = serenity::CreateMessage::new().content(format!(
            "You have been **temporarily banned** from **{guild_name}** for {duration_str}. Reason: {reason}"
        ));
        let _ = user.dm(ctx, dm).await;
    }

    guild_id
        .ban_with_reason(ctx, user.id, delete_days, &reason)
        .await?;

    let user_id = user.id;
    let http = ctx.serenity_context().http.clone();
    let unban_delay = std::time::Duration::from_secs(u64::from(duration) * 60);

    // Spawn the auto-unban task
    tokio::spawn(async move {
        tokio::time::sleep(unban_delay).await;
        if let Err(e) = guild_id.unban(&http, user_id).await {
            tracing::warn!("Failed to auto-unban {} from {guild_id}: {e}", user_id);
        } else {
            tracing::info!("Auto-unbanned {} from {guild_id}", user_id);
        }
    });

    let days = duration / (24 * 60);
    let hours = (duration % (24 * 60)) / 60;
    let mins = duration % 60;
    let duration_display = if days > 0 {
        format!("{days}d {hours}h {mins}m")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{duration}m")
    };

    ctx.say(format!(
        "⏳ Temporarily banned **{}** for **{duration_display}** | Reason: {reason}",
        user.name
    ))
    .await?;
    Ok(())
}
