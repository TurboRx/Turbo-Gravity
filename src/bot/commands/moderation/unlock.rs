use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Unlock the channel and allow messages
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    required_bot_permissions = "MANAGE_CHANNELS"
)]
pub async fn unlock(
    ctx: Context<'_>,
    #[description = "Reason for unlocking the channel"] reason: Option<String>,
) -> Result<(), Error> {
    let reason = reason.as_deref().unwrap_or("No reason provided").to_string();

    // Extract @everyone RoleId before any await
    let everyone_id = {
        let guild = ctx
            .guild()
            .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
        guild.id.everyone_role()
    };

    let channel = ctx
        .channel_id()
        .to_channel(ctx)
        .await?
        .guild()
        .ok_or_else(|| anyhow::anyhow!("Not a guild channel"))?;

    let is_locked = channel
        .permission_overwrites
        .iter()
        .find(|o| o.kind == serenity::PermissionOverwriteType::Role(everyone_id))
        .map(|o| o.deny.contains(serenity::Permissions::SEND_MESSAGES))
        .unwrap_or(false);

    if !is_locked {
        ctx.say("Channel is already unlocked.").await?;
        return Ok(());
    }

    ctx.channel_id()
        .create_permission(
            ctx,
            serenity::PermissionOverwrite {
                allow: serenity::Permissions::SEND_MESSAGES,
                deny: serenity::Permissions::empty(),
                kind: serenity::PermissionOverwriteType::Role(everyone_id),
            },
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to unlock channel: {e}"))?;

    ctx.say(format!("🔓 Channel unlocked. Reason: {reason}"))
        .await?;
    Ok(())
}
