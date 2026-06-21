use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Show the member count breakdown for this server
#[poise::command(slash_command, ephemeral, guild_only)]
pub async fn membercount(ctx: Context<'_>) -> Result<(), Error> {
    // Extract member data from cache before any await
    let (guild_id, guild_name, total, online, bots) = {
        let guild = ctx
            .guild()
            .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

        let total = guild.member_count as usize;
        let bots = guild
            .members
            .values()
            .filter(|m| m.user.bot)
            .count();

        // Count online members from presences (requires GUILD_PRESENCES intent — use 0 if unavailable)
        let online = guild
            .presences
            .values()
            .filter(|p| {
                matches!(
                    p.status,
                    serenity::OnlineStatus::Online
                        | serenity::OnlineStatus::Idle
                        | serenity::OnlineStatus::DoNotDisturb
                )
            })
            .count();

        (guild.id, guild.name.clone(), total, online, bots)
    };

    let humans = total.saturating_sub(bots);

    let embed = serenity::CreateEmbed::new()
        .title(format!("👥 {guild_name} — Member Count"))
        .colour(serenity::Colour::from_rgb(88, 101, 242))
        .field("Total", total.to_string(), true)
        .field("Humans", humans.to_string(), true)
        .field("Bots", bots.to_string(), true)
        .field("Online / Active", online.to_string(), true)
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Server ID: {guild_id}"
        )));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
