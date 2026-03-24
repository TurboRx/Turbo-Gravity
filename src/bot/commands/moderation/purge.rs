use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Bulk delete messages in this channel
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    required_permissions = "MANAGE_MESSAGES",
    required_bot_permissions = "MANAGE_MESSAGES"
)]
pub async fn purge(
    ctx: Context<'_>,
    #[description = "Number of messages to delete (1-100)"]
    #[min = 1_u8]
    #[max = 100_u8]
    amount: u8,
) -> Result<(), Error> {
    let messages = ctx
        .channel_id()
        .messages(ctx, serenity::GetMessages::new().limit(amount))
        .await?;

    let cutoff = serenity::Timestamp::now().unix_timestamp() - 14 * 24 * 60 * 60;
    let ids: Vec<serenity::MessageId> = messages
        .iter()
        .filter(|m| m.timestamp.unix_timestamp() > cutoff)
        .map(|m| m.id)
        .collect();

    if ids.is_empty() {
        ctx.say("No messages to delete (all are older than 14 days).").await?;
        return Ok(());
    }

    let count = ids.len();
    if let Err(e) = ctx.channel_id().delete_messages(ctx, &ids).await {
        tracing::error!("Failed to bulk-delete messages: {e}");
        ctx.say("Failed to delete messages. Please check my permissions.").await?;
        return Ok(());
    }

    ctx.say(format!("Deleted {count} messages.")).await?;
    Ok(())
}
