use crate::bot::{Context, Error};
use rand::RngExt as _;

/// Roll a dice
#[poise::command(slash_command, ephemeral)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "Number of sides on the die (2-1000)"]
    #[min = 2_u32]
    #[max = 1000_u32]
    sides: Option<u32>,
) -> Result<(), Error> {
    let sides = sides.unwrap_or(6);
    let result = rand::rng().random_range(1..=sides);
    ctx.say(format!("🎲 Rolled a {sides}-sided die: **{result}**"))
        .await?;
    Ok(())
}
