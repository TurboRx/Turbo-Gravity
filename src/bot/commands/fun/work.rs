use crate::bot::{Context, Error};
use crate::db::models::User;
use mongodb::bson::DateTime as BsonDateTime;
use poise::serenity_prelude as serenity;
use rand::{seq::IndexedRandom, RngExt as _};

const WORK_COOLDOWN_SECS: i64 = 60 * 60;
const MIN_COINS: i64 = 25;
const MAX_COINS: i64 = 75;
const XP_GAINED: i64 = 5;

const WORK_MESSAGES: &[&str] = &[
    "You delivered packages and earned",
    "You fixed some bugs and earned",
    "You washed dishes and earned",
    "You tutored a student and earned",
    "You walked dogs and earned",
    "You coded a feature and earned",
    "You designed a logo and earned",
    "You drove a taxi and earned",
    "You sold lemonade and earned",
];

/// Work to earn coins (1-hour cooldown)
#[poise::command(slash_command, ephemeral)]
pub async fn work(ctx: Context<'_>) -> Result<(), Error> {
    let Some(db) = ctx.data().database() else {
        ctx.say("Database is unavailable.").await?;
        return Ok(());
    };

    let author = ctx.author();
    let mut profile = User::upsert(
        &db,
        &author.id.to_string(),
        &author.name,
        &author
            .discriminator
            .map(|d| d.to_string())
            .unwrap_or_default(),
        author.avatar_url().as_deref(),
    )
    .await?;

    let now_ms = chrono::Utc::now().timestamp_millis();
    let last_ms = profile.last_work.map_or(0, mongodb::bson::DateTime::timestamp_millis);
    let elapsed_secs = (now_ms - last_ms) / 1000;

    if elapsed_secs < WORK_COOLDOWN_SECS {
        let remaining = WORK_COOLDOWN_SECS - elapsed_secs;
        let minutes = remaining / 60;
        let seconds = remaining % 60;
        ctx.say(format!(
            "⏰ You're tired! Rest for **{minutes}m {seconds}s** before working again."
        ))
        .await?;
        return Ok(());
    }

    let earned = rand::rng().random_range(MIN_COINS..=MAX_COINS);
    let message = WORK_MESSAGES
        .choose(&mut rand::rng())
        .copied()
        .unwrap_or("You worked hard and earned");

    profile.balance += earned;
    profile.xp += XP_GAINED;
    profile.last_work = Some(BsonDateTime::now());

    // Level-up loop: consume XP before incrementing level to prevent infinite leveling
    while profile.xp >= profile.level * 100 {
        profile.xp -= profile.level * 100;
        profile.level += 1;
    }

    profile.save(&db).await?;

    let embed = serenity::CreateEmbed::new()
        .title("💼 Work Complete!")
        .description(format!("{message} **{earned} coins**!"))
        .colour(serenity::Colour::from_rgb(14, 165, 233))
        .field("Balance", profile.balance.to_string(), true)
        .field(
            "XP",
            format!("{} (Level {})", profile.xp, profile.level),
            true,
        )
        .footer(serenity::CreateEmbedFooter::new(
            "Come back in 1 hour to work again.",
        ));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
