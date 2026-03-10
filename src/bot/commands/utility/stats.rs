use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Show bot statistics
#[poise::command(slash_command, ephemeral)]
pub async fn stats(ctx: Context<'_>) -> Result<(), Error> {
    let cache = ctx.cache();
    let guilds = cache.guild_count();

    // Collect member counts without holding any guild CacheRef across await
    let users: usize = cache
        .guilds()
        .iter()
        .map(|g| cache.guild(*g).map(|g| g.member_count as usize).unwrap_or(0))
        .sum();

    let (bot_name, bot_face) = {
        let cu = cache.current_user();
        (cu.name.clone(), cu.face())
    };

    let api_ms = ctx.ping().await.as_millis();

    // Memory usage via /proc/self/statm on Linux (pages × 4 KB → MB)
    let heap_mb = std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .nth(1)
                .and_then(|v| v.parse::<u64>().ok())
        })
        .map(|pages| pages * 4 / 1024)
        .unwrap_or(0);

    let embed = serenity::CreateEmbed::new()
        .title(format!("📊 {bot_name} Statistics"))
        .thumbnail(bot_face)
        .colour(serenity::Colour::from_rgb(88, 101, 242))
        .field("🌐 Servers", guilds.to_string(), true)
        .field("👥 Members", users.to_string(), true)
        .field("📡 API Ping", format!("{api_ms}ms"), true)
        .field("🧠 Memory", format!("{heap_mb} MB"), true);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
