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
    let uid = serenity::UserId::new(uid);

    // Check the ban exists (bans takes http, pagination option, limit option)
    let bans = guild_id.bans(ctx, None, None).await?;
    let ban = bans.iter().find(|b| b.user.id == uid);
    let ban = match ban {
        Some(b) => b.clone(),
        None => {
            ctx.say("That user is not banned.").await?;
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
