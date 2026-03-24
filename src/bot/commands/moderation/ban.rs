use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Ban a user from the server
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS"
)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "User to ban"] user: serenity::User,
    #[description = "Days of messages to delete (0-7)"]
    #[min = 0_u8]
    #[max = 7_u8]
    delete_days: Option<u8>,
    #[description = "Reason for ban"] reason: Option<String>,
) -> Result<(), Error> {
    // Extract non-Send data from the cache guard before any await points
    let (guild_id, guild_name) = {
        let guild = ctx
            .guild()
            .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
        (guild.id, guild.name.clone())
    };

    let reason = reason
        .as_deref()
        .unwrap_or("No reason provided")
        .to_string();
    let delete_days = delete_days.unwrap_or(0);

    if user.id == ctx.author().id {
        ctx.say("You cannot ban yourself.").await?;
        return Ok(());
    }
    if user.id == ctx.cache().current_user().id {
        ctx.say("I cannot ban myself.").await?;
        return Ok(());
    }

    // Check role hierarchy if target is a member of the guild
    if let Ok(target_member) = guild_id.member(ctx, user.id).await {
        let invoker = ctx.author_member().await;
        if let Some(invoker) = invoker {
            let invoker_top = invoker
                .roles(ctx)
                .map(|roles| roles.iter().map(|r| r.position).max().unwrap_or(0))
                .unwrap_or(0);
            let target_top = target_member
                .roles(ctx)
                .map(|roles| roles.iter().map(|r| r.position).max().unwrap_or(0))
                .unwrap_or(0);
            if invoker_top <= target_top {
                ctx.say("You cannot ban someone with an equal or higher role.")
                    .await?;
                return Ok(());
            }
        }

        // Try to DM the target before banning
        let dm = serenity::CreateMessage::new().content(format!(
            "You were banned from **{guild_name}** | Reason: {reason}"
        ));
        let _ = user.dm(ctx, dm).await;
    }

    guild_id
        .ban_with_reason(ctx, user.id, delete_days, &reason)
        .await?;

    ctx.say(format!(
        "Banned {}{}  | Reason: {reason}",
        user.name,
        if delete_days > 0 {
            format!(" and deleted {delete_days} day(s) of messages")
        } else {
            String::new()
        }
    ))
    .await?;
    Ok(())
}
