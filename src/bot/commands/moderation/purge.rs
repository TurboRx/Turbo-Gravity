use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Bulk delete messages in this channel (Discord only allows deleting messages ≤14 days old)
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
    #[description = "Only delete messages from this user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let messages = ctx
        .channel_id()
        .messages(ctx, serenity::GetMessages::new().limit(amount))
        .await?;

    // Discord refuses to bulk-delete messages older than 14 days.
    let cutoff = chrono::Utc::now() - chrono::Duration::days(14);

    let eligible: Vec<serenity::MessageId> = messages
        .iter()
        .filter(|m| {
            // Apply optional user filter
            let user_ok = user.as_ref().map_or(true, |u| m.author.id == u.id);
            // Apply age filter
            let age_ok = m.timestamp.unix_timestamp() > cutoff.timestamp();
            user_ok && age_ok
        })
        .map(|m| m.id)
        .collect();

    let skipped = messages.len().saturating_sub(eligible.len());

    if eligible.is_empty() {
        let reason = if skipped > 0 {
            "All matching messages are older than 14 days and cannot be bulk-deleted."
        } else {
            "No messages found to delete."
        };
        ctx.say(reason).await?;
        return Ok(());
    }

    let count = eligible.len();

    if count == 1 {
        // Bulk delete requires ≥2; fall back to single delete
        eligible[0].delete(ctx.http()).await?;
    } else {
        ctx.channel_id().delete_messages(ctx, &eligible).await?;
    }

    let mut reply = format!("🗑️ Deleted **{count}** message{}.", if count == 1 { "" } else { "s" });
    if skipped > 0 {
        reply.push_str(&format!(
            " *(skipped {skipped} message{} — older than 14 days or filtered out)*",
            if skipped == 1 { "" } else { "s" }
        ));
    }
    ctx.say(reply).await?;
    Ok(())
}
