use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

const NUMBER_EMOJIS: &[&str] = &[
    "1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣", "🔟",
];

/// Create a quick reaction poll
#[poise::command(slash_command)]
pub async fn poll(
    ctx: Context<'_>,
    #[description = "Poll question"] question: String,
    #[description = "Comma-separated choices (2-10)"] choices: String,
) -> Result<(), Error> {
    let options: Vec<&str> = choices
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if options.len() < 2 || options.len() > 10 {
        ctx.say("Please provide between 2 and 10 choices separated by commas.")
            .await?;
        return Ok(());
    }

    let lines: Vec<String> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| format!("{} {opt}", NUMBER_EMOJIS[i]))
        .collect();

    let embed = serenity::CreateEmbed::default()
        .title(format!("📊 {question}"))
        .description(lines.join("\n"))
        .colour(serenity::Colour::from_rgb(245, 158, 11))
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Poll by {}",
            ctx.author().tag()
        )));

    let reply = ctx
        .send(poise::CreateReply::default().embed(embed))
        .await?;

    if let Ok(msg) = reply.message().await {
        for i in 0..options.len() {
            let _ = msg
                .react(
                    ctx.http(),
                    serenity::ReactionType::Unicode(NUMBER_EMOJIS[i].to_string()),
                )
                .await;
        }
    }

    Ok(())
}
