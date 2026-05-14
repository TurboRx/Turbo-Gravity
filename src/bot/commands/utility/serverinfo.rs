use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Show details about this server
#[poise::command(slash_command, ephemeral, guild_only)]
pub async fn serverinfo(ctx: Context<'_>) -> Result<(), Error> {
    // Extract all data we need from the non-Send CacheRef before any await
    let (
        owner_id,
        member_count,
        guild_id,
        guild_name,
        icon_url,
        text_count,
        voice_count,
        created_ts,
    ) = {
        let guild = ctx
            .guild()
            .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
        let text = guild
            .channels
            .values()
            .filter(|c| c.is_text_based())
            .count();
        let voice = guild
            .channels
            .values()
            .filter(|c| {
                matches!(
                    c.kind,
                    serenity::ChannelType::Voice | serenity::ChannelType::Stage
                )
            })
            .count();
        (
            guild.owner_id,
            guild.member_count,
            guild.id,
            guild.name.clone(),
            guild.icon_url(),
            text,
            voice,
            guild.id.created_at().unix_timestamp(),
        )
    };

    // Now safe to await
    let owner = owner_id.to_user(ctx).await.ok();

    let mut embed = serenity::CreateEmbed::new()
        .title(&guild_name)
        .colour(serenity::Colour::from_rgb(0, 168, 132))
        .field("Server ID", guild_id.to_string(), true)
        .field(
            "Owner",
            owner
                .as_ref().map_or_else(|| "Unknown".into(), |u| u.name.clone()),
            true,
        )
        .field("Members", member_count.to_string(), true)
        .field(
            "Channels",
            format!("{text_count} text | {voice_count} voice"),
            true,
        )
        .field("Created", format!("<t:{created_ts}:F>"), true);

    if let Some(icon) = icon_url {
        embed = embed.thumbnail(icon);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
