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
    let (guild_id, guild_name, bot_top, invoker_top, target_top) = {
        let guild = ctx
            .guild()
            .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

        let bot_id = ctx.cache().current_user().id;

        let role_top = |member: &serenity::Member| {
            guild
                .member_highest_role(member)
                .map(|r| r.position)
                .unwrap_or(0)
        };

        let bot_top = guild
            .members
            .get(&bot_id)
            .map(|m| role_top(m))
            .unwrap_or(0);

        let invoker_top = guild
            .members
            .get(&ctx.author().id)
            .map(|m| role_top(m))
            .unwrap_or(0);

        let target_top = guild
            .members
            .get(&user.id)
            .map(|m| role_top(m))
            .unwrap_or(0);

        (guild.id, guild.name.clone(), bot_top, invoker_top, target_top)
    };

    let reason = reason.as_deref().unwrap_or("No reason provided").to_string();
    let delete_days = delete_days.unwrap_or(0);

    if user.id == ctx.author().id {
        ctx.say("You cannot ban yourself.").await?;
        return Ok(());
    }
    if user.id == ctx.cache().current_user().id {
        ctx.say("I cannot ban myself.").await?;
        return Ok(());
    }

    if invoker_top <= target_top {
        ctx.say("You cannot ban someone with an equal or higher role.")
            .await?;
        return Ok(());
    }

    if bot_top <= target_top {
        ctx.say("I cannot ban that user — they have an equal or higher role than me.")
            .await?;
        return Ok(());
    }

    // Try to DM the target before banning
    let dm = serenity::CreateMessage::new()
        .content(format!("You were banned from **{guild_name}** | Reason: {reason}"));
    let _ = user.dm(ctx, dm).await;

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
