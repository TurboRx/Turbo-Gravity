use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Read RSS memory usage from /proc/self/statm (Linux only).
/// Returns megabytes, or None on non-Linux platforms.
fn resident_memory_mb() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/self/statm")
            .ok()
            .and_then(|s| {
                s.split_whitespace()
                    .nth(1) // RSS field
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .map(|pages| pages * 4 / 1024) // 4 KiB pages → MiB
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Show bot statistics
#[poise::command(slash_command, ephemeral)]
pub async fn stats(ctx: Context<'_>) -> Result<(), Error> {
    let cache = ctx.cache();
    let guilds = cache.guild_count();

    // Collect member counts without holding any guild CacheRef across await
    let users: usize = cache
        .guilds()
        .iter()
        .map(|g| {
            cache
                .guild(*g)
                .map(|g| g.member_count as usize)
                .unwrap_or(0)
        })
        .sum();

    let (bot_name, bot_face) = {
        let cu = cache.current_user();
        (cu.name.clone(), cu.face())
    };

    let api_ms = ctx.ping().await.as_millis();

    let mut embed = serenity::CreateEmbed::new()
        .title(format!("📊 {bot_name} Statistics"))
        .thumbnail(bot_face)
        .colour(serenity::Colour::from_rgb(88, 101, 242))
        .field("🌐 Servers", guilds.to_string(), true)
        .field("👥 Members", users.to_string(), true)
        .field("📡 API Ping", format!("{api_ms}ms"), true);

    if let Some(mb) = resident_memory_mb() {
        embed = embed.field("🧠 Memory", format!("{mb} MB"), true);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
