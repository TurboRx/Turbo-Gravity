use crate::bot::{Context, Error};
use crate::db::models::Warning;
use poise::serenity_prelude as serenity;

/// Remove a specific warning by its number (use /warnings to find numbers)
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn clearwarn(
    ctx: Context<'_>,
    #[description = "Member whose warning to remove"] user: serenity::User,
    #[description = "Warning number to remove (1-based, from /warnings)"]
    #[min = 1_u32]
    warning_number: u32,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?
        .to_string();

    let Some(db) = ctx.data().database() else {
        ctx.say("Database is unavailable.").await?;
        return Ok(());
    };

    let total = Warning::count(&db, &guild_id, &user.id.to_string()).await?;
    if total == 0 {
        ctx.say(format!("{} has no warnings in this server.", user.name))
            .await?;
        return Ok(());
    }

    let idx = u64::from(warning_number).saturating_sub(1);
    if idx >= total {
        ctx.say(format!(
            "Warning #{warning_number} does not exist. {} has {} warning{}.",
            user.name,
            total,
            if total == 1 { "" } else { "s" }
        ))
        .await?;
        return Ok(());
    }

    // Fetch the specific warning (sorted newest-first, same as /warnings)
    let warnings = Warning::find_paginated(&db, &guild_id, &user.id.to_string(), idx, 1).await?;

    let Some(warning) = warnings.into_iter().next() else {
        ctx.say("Could not find that warning.").await?;
        return Ok(());
    };

    let warning_id = warning
        .id
        .ok_or_else(|| anyhow::anyhow!("Warning has no database ID"))?;

    // Delete by ObjectId
    let col = Warning::collection(&db);
    col.delete_one(mongodb::bson::doc! { "_id": warning_id })
        .await?;

    ctx.say(format!(
        "✅ Removed warning #{warning_number} from {} (reason was: *{}*).",
        user.name, warning.reason
    ))
    .await?;
    Ok(())
}
