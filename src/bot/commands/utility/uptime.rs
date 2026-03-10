use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

fn format_duration(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    parts.push(format!("{seconds}s"));
    parts.join(" ")
}

/// Show bot process and connection uptime
#[poise::command(slash_command, ephemeral)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let api_ms = ctx.ping().await.as_millis();

    // Process uptime from /proc/uptime
    let process_secs = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .and_then(|v| v.parse::<f64>().ok())
        })
        .map(|s| s as u64)
        .unwrap_or(0);

    let embed = serenity::CreateEmbed::new()
        .title("Uptime")
        .colour(serenity::Colour::from_rgb(34, 197, 94))
        .field("Process", format_duration(process_secs), true)
        .field("API Ping", format!("{api_ms}ms"), true);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
