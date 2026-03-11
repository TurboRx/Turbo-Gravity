use crate::bot::{Context, Error};
use crate::db::models::Warning;
use poise::serenity_prelude as serenity;

/// Send a formal warning to a member
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn warn(
    ctx: Context<'_>,
    #[description = "Member to warn"] member: serenity::Member,
    #[description = "Reason for warning"] reason: String,
) -> Result<(), Error> {
    if member.user.id == ctx.author().id {
        ctx.say("You cannot warn yourself.").await?;
        return Ok(());
    }
    if member.user.id == ctx.cache().current_user().id {
        ctx.say("I cannot warn myself.").await?;
        return Ok(());
    }

    // Extract guild name before any await (CacheRef is !Send)
    let (guild_id, guild_name) = {
        let guild = ctx
            .guild()
            .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
        (guild.id.to_string(), guild.name.clone())
    };

    let warn_count = if let Some(db) = ctx.data().database() {
        Warning::create(
            &db,
            &guild_id,
            &member.user.id.to_string(),
            &ctx.author().id.to_string(),
            &reason,
        )
        .await
        .ok();
        Warning::count(&db, &guild_id, &member.user.id.to_string())
            .await
            .ok()
    } else {
        None
    };

    // Try to DM the target
    let dm_embed = serenity::CreateEmbed::new()
        .title(format!("⚠️ Warning from {guild_name}"))
        .colour(serenity::Colour::from_rgb(245, 158, 11))
        .field("Reason", &reason, false);
    let _ = member
        .user
        .dm(ctx, serenity::CreateMessage::new().embed(dm_embed))
        .await;

    let mut embed = serenity::CreateEmbed::new()
        .title("⚠️ Warning Issued")
        .colour(serenity::Colour::from_rgb(245, 158, 11))
        .field(
            "Member",
            format!("{} ({})", member.user.name, member.user.id),
            true,
        )
        .field("Moderator", ctx.author().name.clone(), true)
        .field("Reason", &reason, false);

    if let Some(count) = warn_count {
        embed = embed.footer(serenity::CreateEmbedFooter::new(format!(
            "Total warnings for this user: {count}"
        )));
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
