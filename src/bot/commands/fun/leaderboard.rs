use crate::bot::{Context, Error};
use crate::db::models::User;
use poise::serenity_prelude as serenity;

/// Show the top 10 richest users in the economy
#[poise::command(slash_command, ephemeral)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let Some(db) = ctx.data().database() else {
        ctx.say("Database is unavailable.").await?;
        return Ok(());
    };

    use mongodb::options::FindOptions;
    let col = User::collection(&db);
    let opts = FindOptions::builder()
        .sort(mongodb::bson::doc! { "balance": -1_i32 })
        .limit(10)
        .build();

    let mut cursor = col.find(mongodb::bson::doc! {}).with_options(opts).await?;
    let mut users = Vec::new();
    while cursor.advance().await? {
        users.push(cursor.deserialize_current()?);
    }

    if users.is_empty() {
        ctx.say("No economy profiles exist yet. Use `/daily` or `/work` to get started!")
            .await?;
        return Ok(());
    }

    let mut description = String::new();
    let medals = ["🥇", "🥈", "🥉"];
    for (i, u) in users.iter().enumerate() {
        let rank = medals.get(i).copied().unwrap_or("🔹");
        description.push_str(&format!(
            "{}  **{}** — {} coins | Level {}\n",
            rank, u.username, u.balance, u.level
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title("🏆 Economy Leaderboard")
        .description(description)
        .colour(serenity::Colour::from_rgb(245, 158, 11))
        .footer(serenity::CreateEmbedFooter::new("Top 10 by coin balance"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
