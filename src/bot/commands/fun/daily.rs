use crate::bot::{Context, Error};
use crate::db::models::User;
use mongodb::bson::DateTime as BsonDateTime;
use poise::serenity_prelude as serenity;
use rand::RngExt as _;

const DAILY_COOLDOWN_SECS: i64 = 24 * 60 * 60;
const MIN_COINS: i64 = 100;
const MAX_COINS: i64 = 200;
const XP_GAINED: i64 = 10;

/// Claim your daily coin reward
#[poise::command(slash_command, ephemeral)]
pub async fn daily(ctx: Context<'_>) -> Result<(), Error> {
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
    let last_ms = profile
        .last_daily
        .map_or(0, mongodb::bson::DateTime::timestamp_millis);
    // Clamp to 0 to guard against clock skew or future timestamps in the DB.
    let elapsed_secs = ((now_ms - last_ms) / 1000).max(0);

    if elapsed_secs < DAILY_COOLDOWN_SECS {
        let remaining = DAILY_COOLDOWN_SECS - elapsed_secs;
        let hours = remaining / 3600;
        let minutes = (remaining % 3600) / 60;
        let seconds = remaining % 60;
        let time_str = if hours > 0 {
            format!("{hours}h {minutes}m")
        } else if minutes > 0 {
            format!("{minutes}m {seconds}s")
        } else {
            format!("{seconds}s")
        };
        ctx.say(format!(
            "⏰ Daily already claimed! Come back in **{time_str}**."
        ))
        .await?;
        return Ok(());
    }

    let earned = rand::rng().random_range(MIN_COINS..=MAX_COINS);
    profile.balance += earned;
    profile.xp += XP_GAINED;
    profile.last_daily = Some(BsonDateTime::now());

    // Guard against a corrupted level value — a level of 0 would cause the
    // XP cost to be 0 and create an infinite loop.
    if profile.level < 1 {
        profile.level = 1;
    }

    // Level-up loop: consume XP before incrementing level to prevent infinite leveling
    while profile.xp >= profile.level * 100 {
        profile.xp -= profile.level * 100;
        profile.level += 1;
    }

    profile.save(&db).await?;

    let embed = serenity::CreateEmbed::new()
        .title("💰 Daily Reward Claimed!")
        .colour(serenity::Colour::from_rgb(34, 197, 94))
        .field("Earned", format!("+{earned} coins"), true)
        .field("Balance", profile.balance.to_string(), true)
        .field(
            "XP",
            format!("{} (Level {})", profile.xp, profile.level),
            true,
        )
        .footer(serenity::CreateEmbedFooter::new(
            "Come back tomorrow for more!",
        ));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
