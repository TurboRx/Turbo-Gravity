use crate::bot::{Context, Error};

/// Show current Discord connection time
#[poise::command(slash_command, ephemeral)]
pub async fn contime(ctx: Context<'_>) -> Result<(), Error> {
    let shard_info = ctx.serenity_context().shard_id;
    let ping = ctx.ping().await.as_millis();
    ctx.say(format!("Shard {shard_info} | API ping: {ping}ms"))
        .await?;
    Ok(())
}
