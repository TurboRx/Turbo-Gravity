use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;
use std::time::Duration;

/// Set a one-time reminder
#[poise::command(slash_command, ephemeral)]
pub async fn remind(
    ctx: Context<'_>,
    #[description = "What to remind you about"] message: String,
    #[description = "Minutes until reminder (1-10080)"]
    #[min = 1_u32]
    #[max = 10080_u32]
    minutes: u32,
) -> Result<(), Error> {
    if message.len() > 1000 {
        ctx.say("Reminder message is too long (max 1000 characters).")
            .await?;
        return Ok(());
    }

    ctx.say(format!(
        "⏰ Reminder set for {minutes} minute(s). I'll DM you when it's time."
    ))
    .await?;

    let user = ctx.author().clone();
    let http = ctx.serenity_context().http.clone();
    let delay = Duration::from_secs(u64::from(minutes) * 60);

    tokio::spawn(async move {
        tokio::time::sleep(delay).await;
        let dm = serenity::CreateMessage::new().content(format!("⏰ **Reminder:** {message}"));
        let _ = user.dm(&http, dm).await;
    });

    Ok(())
}
