use crate::bot::{Context, Error};

/// Flip a coin
#[poise::command(slash_command, ephemeral)]
pub async fn coinflip(ctx: Context<'_>) -> Result<(), Error> {
    let result = if rand::random::<bool>() { "Heads" } else { "Tails" };
    ctx.say(format!("🪙 The coin landed on **{result}**."))
        .await?;
    Ok(())
}
