use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;
use rand::seq::IndexedRandom;

/// Host a giveaway in this channel
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES"
)]
pub async fn giveaway(
    ctx: Context<'_>,
    #[description = "What to give away"] prize: String,
    #[description = "Duration in minutes (1-10080)"]
    #[min = 1_u32]
    #[max = 10080_u32]
    duration: u32,
    #[description = "Number of winners (1-10)"]
    #[min = 1_u8]
    #[max = 10_u8]
    winners: Option<u8>,
) -> Result<(), Error> {
    let winners = winners.unwrap_or(1);
    let end_ts = chrono::Utc::now().timestamp() + i64::from(duration) * 60;

    let embed = serenity::CreateEmbed::new()
        .title("🎉 GIVEAWAY 🎉")
        .description(format!(
            "**Prize:** {prize}\n\nReact with 🎉 to enter!\n\n**Ends:** <t:{end_ts}:R> (<t:{end_ts}:f>)\n**Winners:** {winners}\n**Hosted by:** {}",
            ctx.author().name
        ))
        .colour(serenity::Colour::from_rgb(255, 215, 0))
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Ends at • {winners} winner{}", if winners == 1 { "" } else { "s" }
        )));

    // Send the giveaway message to the channel (not ephemeral so everyone sees it)
    let msg = ctx
        .channel_id()
        .send_message(ctx, serenity::CreateMessage::new().embed(embed))
        .await?;

    // Add the entry reaction
    msg.react(
        ctx.http(),
        serenity::ReactionType::Unicode("🎉".to_string()),
    )
    .await?;

    // Confirm to the command invoker
    ctx.say(format!(
        "✅ Giveaway started! It will end <t:{end_ts}:R>."
    ))
    .await?;

    let msg_id = msg.id;
    let channel_id = ctx.channel_id();
    let http = ctx.serenity_context().http.clone();
    let cache = ctx.serenity_context().cache.clone();
    let prize_clone = prize.clone();
    let delay = std::time::Duration::from_secs(u64::from(duration) * 60);

    tokio::spawn(async move {
        tokio::time::sleep(delay).await;

        // Fetch all users who reacted with 🎉
        let reaction_type = serenity::ReactionType::Unicode("🎉".to_string());
        let reactors = channel_id
            .reaction_users(&http, msg_id, reaction_type, Some(100), None)
            .await
            .unwrap_or_default();

        // Filter out bots
        let bot_id = cache.current_user().id;
        let mut eligible: Vec<_> = reactors
            .into_iter()
            .filter(|u| !u.bot && u.id != bot_id)
            .collect();

        // Pick winners
        let mut rng = rand::rng();
        let mut picked: Vec<String> = Vec::new();
        let num_winners = usize::from(winners).min(eligible.len());
        for _ in 0..num_winners {
            if eligible.is_empty() {
                break;
            }
            let idx = (rng.random_range(0_u32..eligible.len() as u32)) as usize;
            let winner = eligible.remove(idx);
            picked.push(format!("<@{}>", winner.id));
        }

        let result_embed = serenity::CreateEmbed::new()
            .title("🎉 Giveaway Ended!")
            .colour(serenity::Colour::from_rgb(255, 215, 0));

        let announcement = if picked.is_empty() {
            let embed = result_embed.description(format!(
                "**Prize:** {prize_clone}\n\nNo valid entries — no winner selected."
            ));
            channel_id
                .send_message(&http, serenity::CreateMessage::new().embed(embed))
                .await
        } else {
            let winners_str = picked.join(", ");
            let embed = result_embed.description(format!(
                "**Prize:** {prize_clone}\n\n🏆 **Winner{}: {}**\n\nCongratulations!",
                if picked.len() == 1 { "" } else { "s" },
                winners_str
            ));
            channel_id
                .send_message(&http, serenity::CreateMessage::new().embed(embed))
                .await
        };

        if let Err(e) = announcement {
            tracing::warn!("Failed to announce giveaway result: {e}");
        }
    });

    Ok(())
}
