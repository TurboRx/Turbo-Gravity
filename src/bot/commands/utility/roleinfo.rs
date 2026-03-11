use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// View details about a role
#[poise::command(slash_command, ephemeral, guild_only)]
pub async fn roleinfo(
    ctx: Context<'_>,
    #[description = "Role to inspect"] role: serenity::Role,
) -> Result<(), Error> {
    let created_ts = role.id.created_at().unix_timestamp();
    let colour = role.colour;

    let embed = serenity::CreateEmbed::new()
        .title(&role.name)
        .colour(colour)
        .field("ID", role.id.to_string(), true)
        .field("Color", format!("#{:06X}", colour.0), true)
        .field(
            "Mentionable",
            if role.mentionable { "Yes" } else { "No" },
            true,
        )
        .field("Hoisted", if role.hoist { "Yes" } else { "No" }, true)
        .field("Position", role.position.to_string(), true)
        .field("Created", format!("<t:{created_ts}:R>"), true);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
