use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Kick a member from the server
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    required_permissions = "KICK_MEMBERS",
    required_bot_permissions = "KICK_MEMBERS"
)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "Member to kick"] member: serenity::Member,
    #[description = "Reason for kick"] reason: Option<String>,
) -> Result<(), Error> {
    // Extract guild name before any await (CacheRef is !Send)
    let guild_name = {
        let guild = ctx
            .guild()
            .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
        guild.name.clone()
    };
    let reason = reason
        .as_deref()
        .unwrap_or("No reason provided")
        .to_string();

    if member.user.id == ctx.author().id {
        ctx.say("You cannot kick yourself.").await?;
        return Ok(());
    }
    if member.user.id == ctx.cache().current_user().id {
        ctx.say("I cannot kick myself.").await?;
        return Ok(());
    }

    // Role hierarchy check
    let invoker = ctx.author_member().await;
    if let Some(invoker) = invoker {
        let invoker_top = invoker
            .roles(ctx)
            .map(|roles| roles.iter().map(|r| r.position).max().unwrap_or(0))
            .unwrap_or(0);
        let target_top = member
            .roles(ctx)
            .map(|roles| roles.iter().map(|r| r.position).max().unwrap_or(0))
            .unwrap_or(0);
        if invoker_top <= target_top {
            ctx.say("You cannot kick someone with an equal or higher role.")
                .await?;
            return Ok(());
        }
    }

    // Try to DM before kicking
    let dm = serenity::CreateMessage::new().content(format!(
        "You were kicked from **{guild_name}** | Reason: {reason}"
    ));
    let _ = member.user.dm(ctx, dm).await;

    member.kick_with_reason(ctx, &reason).await?;
    ctx.say(format!("Kicked {} | Reason: {reason}", member.user.name))
        .await?;
    Ok(())
}
