use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Timeout a member
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MODERATE_MEMBERS"
)]
pub async fn timeout(
    ctx: Context<'_>,
    #[description = "Member to timeout"] mut member: serenity::Member,
    #[description = "Duration in minutes (1-40320)"]
    #[min = 1_u32]
    #[max = 40320_u32]
    duration: u32,
    #[description = "Reason for timeout"] reason: Option<String>,
) -> Result<(), Error> {
    let reason = reason.as_deref().unwrap_or("No reason provided");

    if member.user.id == ctx.author().id {
        ctx.say("You cannot timeout yourself.").await?;
        return Ok(());
    }
    if member.user.id == ctx.cache().current_user().id {
        ctx.say("I cannot timeout myself.").await?;
        return Ok(());
    }

    // Role hierarchy check — extract positions before any await
    let invoker = ctx.author_member().await;
    if let Some(invoker) = invoker {
        let invoker_top = invoker
            .roles(ctx)
            .map_or(0, |roles| roles.iter().map(|r| r.position).max().unwrap_or(0));
        let target_top = member
            .roles(ctx)
            .map_or(0, |roles| roles.iter().map(|r| r.position).max().unwrap_or(0));
        if invoker_top <= target_top {
            ctx.say("You cannot timeout someone with an equal or higher role.")
                .await?;
            return Ok(());
        }
    }

    // Build a Timestamp from current time + duration
    let until_secs = chrono::Utc::now().timestamp() + i64::from(duration) * 60;
    let until = serenity::Timestamp::from_unix_timestamp(until_secs)
        .map_err(|e| anyhow::anyhow!("Invalid timestamp: {e}"))?;

    member
        .disable_communication_until_datetime(ctx, until)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to timeout member: {e}"))?;

    ctx.say(format!(
        "Timed out {} for {duration} minutes | Reason: {reason}",
        member.user.name
    ))
    .await?;
    Ok(())
}
