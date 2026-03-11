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

    let ids: Vec<serenity::MessageId> = messages.iter().map(|m| m.id).collect();
    let count = ids.len();

    ctx.channel_id().delete_messages(ctx, &ids).await?;

    ctx.say(format!("Deleted {count} messages.")).await?;
    Ok(())
}
