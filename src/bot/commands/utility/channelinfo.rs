use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// View details about a channel
#[poise::command(slash_command, ephemeral, guild_only)]
pub async fn channelinfo(
    ctx: Context<'_>,
    #[description = "Channel to inspect"] channel: Option<serenity::GuildChannel>,
) -> Result<(), Error> {
    // Resolve: use provided channel or fetch current channel
    let ch = match channel {
        Some(c) => c,
        None => ctx
            .channel_id()
            .to_channel(ctx)
            .await?
            .guild()
            .ok_or_else(|| anyhow::anyhow!("Not a guild channel"))?,
    };

    let type_name = match ch.kind {
        serenity::ChannelType::Text => "Text",
        serenity::ChannelType::Voice => "Voice",
        serenity::ChannelType::Category => "Category",
        serenity::ChannelType::News => "Announcement",
        serenity::ChannelType::Stage => "Stage",
        serenity::ChannelType::Forum => "Forum",
        _ => "Unknown",
    };

    let created_ts = ch.id.created_at().unix_timestamp();

    let mut embed = serenity::CreateEmbed::new()
        .title(format!("#{}", ch.name))
        .colour(serenity::Colour::from_rgb(88, 101, 242))
        .field("ID", ch.id.to_string(), true)
        .field("Type", type_name, true)
        .field("Created", format!("<t:{created_ts}:R>"), true);

    if let Some(topic) = &ch.topic {
        if !topic.is_empty() {
            embed = embed.field("Topic", topic.clone(), false);
        }
    }
    if ch.rate_limit_per_user.unwrap_or(0) > 0 {
        embed = embed.field(
            "Slowmode",
            format!("{}s", ch.rate_limit_per_user.unwrap()),
            true,
        );
    }
    if let Some(parent) = ch.parent_id {
        embed = embed.field("Category", format!("<#{parent}>"), true);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
