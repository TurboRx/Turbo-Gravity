use crate::bot::{Context, Error};
use crate::db::models::{bson_dt_to_chrono, Warning};
use poise::serenity_prelude as serenity;

const PER_PAGE: u64 = 5;

/// View warnings for a member
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn warnings(
    ctx: Context<'_>,
    #[description = "Member to check"] user: serenity::User,
    #[description = "Page number"]
    #[min = 1_u32]
    page: Option<u32>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?
        .to_string();
    let Some(db) = ctx.data().database() else {
        ctx.say("Database is unavailable. Warnings cannot be retrieved.")
            .await?;
        return Ok(());
    };

    let page = page.unwrap_or(1).saturating_sub(1) as u64;
    let total = Warning::count(&db, &guild_id, &user.id.to_string()).await?;

    if total == 0 {
        ctx.say(format!("{} has no warnings in this server.", user.name))
            .await?;
        return Ok(());
    }

    let warnings = Warning::find_paginated(
        &db,
        &guild_id,
        &user.id.to_string(),
        page * PER_PAGE,
        PER_PAGE as i64,
    )
    .await?;

    let total_pages = total.div_ceil(PER_PAGE);
    let mut embed = serenity::CreateEmbed::new()
        .title(format!("⚠️ Warnings for {}", user.name))
        .colour(serenity::Colour::from_rgb(245, 158, 11))
        .thumbnail(user.face())
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Total: {total} warning{} | Page {} of {total_pages}",
            if total != 1 { "s" } else { "" },
            page + 1
        )));

    if warnings.is_empty() {
        embed = embed.description("No warnings on this page.");
    } else {
        for (i, w) in warnings.iter().enumerate() {
            let ts = bson_dt_to_chrono(w.created_at).timestamp();
            embed = embed.field(
                format!("#{} — <t:{ts}:R>", page * PER_PAGE + i as u64 + 1),
                format!(
                    "**Reason:** {}\n**Moderator:** <@{}>",
                    w.reason, w.moderator_id
                ),
                false,
            );
        }
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
