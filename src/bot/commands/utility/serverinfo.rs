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
        role_count,
        emoji_count,
        sticker_count,
        boost_count,
        boost_tier,
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

        let boost_tier = match guild.premium_tier {
            serenity::PremiumTier::Tier1 => "Tier 1",
            serenity::PremiumTier::Tier2 => "Tier 2",
            serenity::PremiumTier::Tier3 => "Tier 3",
            _ => "None",
        };

        (
            guild.owner_id,
            guild.member_count,
            guild.id,
            guild.name.clone(),
            guild.icon_url(),
            text,
            voice,
            guild.roles.len(),
            guild.emojis.len(),
            guild.stickers.len(),
            guild.premium_subscription_count.unwrap_or(0),
            boost_tier,
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
                .as_ref()
                .map(|u| u.name.clone())
                .unwrap_or_else(|| "Unknown".into()),
            true,
        )
        .field("Members", member_count.to_string(), true)
        .field(
            "Channels",
            format!("{text_count} text | {voice_count} voice"),
            true,
        )
        .field("Roles", role_count.to_string(), true)
        .field("Emojis", emoji_count.to_string(), true)
        .field("Stickers", sticker_count.to_string(), true)
        .field("Boost Tier", boost_tier, true)
        .field("Boosts", boost_count.to_string(), true)
        .field("Created", format!("<t:{created_ts}:F>"), true);

    if let Some(icon) = icon_url {
        embed = embed.thumbnail(icon);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
