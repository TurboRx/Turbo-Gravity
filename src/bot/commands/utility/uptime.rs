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

/// Read system uptime in seconds from /proc/uptime (Linux only)
fn system_uptime_secs() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|s| {
                s.split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<f64>().ok())
            })
            .map(|s| s as u64)
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Show bot process and connection uptime
#[poise::command(slash_command, ephemeral)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let api_ms = ctx.ping().await.as_millis();

    let mut embed = serenity::CreateEmbed::new()
        .title("Uptime")
        .colour(serenity::Colour::from_rgb(34, 197, 94))
        .field("📡 API Ping", format!("{api_ms}ms"), true);

    if let Some(secs) = system_uptime_secs() {
        embed = embed.field("⏱ System Uptime", format_duration(secs), true);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3600), "1h 0s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
        assert_eq!(format_duration(86400), "1d 0s");
        assert_eq!(format_duration(86400 + 3600 + 60 + 1), "1d 1h 1m 1s");
    }
}
