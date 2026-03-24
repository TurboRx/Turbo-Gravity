use crate::bot::{Context, Error};

/// Check bot and API latency
#[poise::command(slash_command, ephemeral)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = std::time::Instant::now();
    let msg = ctx.say("Pinging…").await?;
    let roundtrip_ms = start.elapsed().as_millis();
    let api_ms = ctx.ping().await.as_millis();

    msg.edit(
        ctx,
        poise::CreateReply::default().content(format!(
            "🏓 Pong! Roundtrip: {roundtrip_ms}ms | API ping: {api_ms}ms"
        )),
    )
    .await?;
    Ok(())
}
