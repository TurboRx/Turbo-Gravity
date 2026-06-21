use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Unban a user from the server
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS"
)]
pub async fn unban(
    ctx: Context<'_>,
    #[description = "Discord User ID to unban"] user_id: String,
    #[description = "Reason for unban"] reason: Option<String>,
) -> Result<(), Error> {
    let reason = reason.as_deref().unwrap_or("No reason provided");
    // Extract guild_id (Copy) before any await
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    let uid: u64 = user_id
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid user ID – must be a numeric Discord snowflake"))?;

    // Snowflake 0 is never a real user
    if uid == 0 {
        ctx.say("Invalid user ID.").await?;
        return Ok(());
    }

    let uid = serenity::UserId::new(uid);

    // Look up the specific ban entry directly instead of fetching all bans
    // (fetching all bans silently fails on servers with > 1000 bans).
    let ban = match guild_id.ban(ctx, uid).await {
        Ok(b) => b,
        Err(_) => {
            ctx.say("That user is not banned on this server.").await?;
            return Ok(());
        }
    };

    guild_id
        .unban(ctx, uid)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to unban: {e}"))?;

    ctx.say(format!("Unbanned {} | Reason: {reason}", ban.user.name))
        .await?;
    Ok(())
}
