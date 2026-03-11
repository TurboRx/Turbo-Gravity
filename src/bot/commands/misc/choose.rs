use crate::bot::{Context, Error};
use rand::seq::SliceRandom;

/// Let the bot pick from your options
#[poise::command(slash_command, ephemeral)]
pub async fn choose(
    ctx: Context<'_>,
    #[description = "Comma or pipe-separated choices (min 2)"] options: String,
    #[description = "Optional question/context"] question: Option<String>,
) -> Result<(), Error> {
    let choices: Vec<&str> = options
        .split([',', '|'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if choices.len() < 2 {
        ctx.say("Please provide at least two distinct options separated by commas or pipes.")
            .await?;
        return Ok(());
    }

    if choices.len() > 25 {
        ctx.say("Too many options! Please provide 25 or fewer choices.")
            .await?;
        return Ok(());
    }

    let choice = choices
        .choose(&mut rand::thread_rng())
        .copied()
        .unwrap_or("¯\\_(ツ)_/¯");

    let prompt = question
        .map(|q| format!("**{q}**\n"))
        .unwrap_or_default();

    ctx.say(format!("{prompt}🎲 I choose: **{choice}**")).await?;
    Ok(())
}
