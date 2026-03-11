use crate::bot::{Context, Error};
use rand::seq::IndexedRandom;

const RESPONSES: &[&str] = &[
    "It is certain.",
    "It is decidedly so.",
    "Without a doubt.",
    "Yes — definitely.",
    "You may rely on it.",
    "As I see it, yes.",
    "Most likely.",
    "Outlook good.",
    "Yes.",
    "Signs point to yes.",
    "Reply hazy, try again.",
    "Ask again later.",
    "Better not tell you now.",
    "Cannot predict now.",
    "Concentrate and ask again.",
    "Don't count on it.",
    "My reply is no.",
    "My sources say no.",
    "Outlook not so good.",
    "Very doubtful.",
];

/// Ask the magic 8-ball a question
#[poise::command(slash_command, ephemeral)]
pub async fn ball8(
    ctx: Context<'_>,
    #[description = "Your question"] question: String,
) -> Result<(), Error> {
    let answer = RESPONSES
        .choose(&mut rand::rng())
        .copied()
        .unwrap_or("Maybe.");
    ctx.say(format!("🎱 **{}**\n{}", question, answer)).await?;
    Ok(())
}
