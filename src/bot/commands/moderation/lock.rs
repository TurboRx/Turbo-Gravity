use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Lock the channel to stop new messages
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    required_bot_permissions = "MANAGE_CHANNELS"
)]
pub async fn lock(
    ctx: Context<'_>,
    #[description = "Reason for locking the channel"] reason: Option<String>,
) -> Result<(), Error> {
    let reason = reason
        .as_deref()
        .unwrap_or("No reason provided")
        .to_string();

    // Extract @everyone RoleId before any await (CacheRef is !Send)
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

    // Check if already locked
    if let Some(overwrite) = channel
        .permission_overwrites
        .iter()
        .find(|o| o.kind == serenity::PermissionOverwriteType::Role(everyone_id))
    {
        if overwrite
            .deny
            .contains(serenity::Permissions::SEND_MESSAGES)
        {
            ctx.say("Channel is already locked.").await?;
            return Ok(());
        }
    }

    ctx.channel_id()
        .create_permission(
            ctx,
            serenity::PermissionOverwrite {
                allow: serenity::Permissions::empty(),
                deny: serenity::Permissions::SEND_MESSAGES,
                kind: serenity::PermissionOverwriteType::Role(everyone_id),
            },
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to lock channel: {e}"))?;

    ctx.say(format!("🔒 Channel locked. Reason: {reason}"))
        .await?;
    Ok(())
}
