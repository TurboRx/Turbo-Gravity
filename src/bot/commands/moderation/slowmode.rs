use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Set channel slowmode
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    required_bot_permissions = "MANAGE_CHANNELS"
)]
pub async fn slowmode(
    ctx: Context<'_>,
    #[description = "Slowmode duration in seconds (0 to disable)"]
    #[min = 0_u16]
    #[max = 21600_u16]
    seconds: u16,
    #[description = "Reason for slowmode change"] reason: Option<String>,
) -> Result<(), Error> {
    let reason = reason.as_deref().unwrap_or("No reason provided");

    ctx.channel_id()
        .edit(
            ctx,
            serenity::EditChannel::new()
                .rate_limit_per_user(seconds)
                .audit_log_reason(reason),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to update slowmode: {e}"))?;

    let msg = if seconds == 0 {
        "Slowmode disabled for this channel.".to_string()
    } else {
        format!("Slowmode set to {seconds} second(s). Reason: {reason}")
    };
    ctx.say(msg).await?;
    Ok(())
}
