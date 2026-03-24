use crate::bot::{Context, Error};
use crate::db::models::User;
use poise::serenity_prelude as serenity;

/// Check your coin balance and XP
#[poise::command(slash_command, ephemeral)]
pub async fn balance(
    ctx: Context<'_>,
    #[description = "User to check (defaults to yourself)"] target: Option<serenity::User>,
) -> Result<(), Error> {
    let user = target.as_ref().unwrap_or_else(|| ctx.author());

    let Some(db) = ctx.data().database() else {
        ctx.say("Database is unavailable.").await?;
        return Ok(());
    };

    let profile = User::find_by_discord_id(&db, &user.id.to_string()).await?;

    match profile {
        None => {
            let is_self = user.id == ctx.author().id;
            let msg = if is_self {
                "You have no profile yet. Use `/daily` or `/work` to get started!".to_string()
            } else {
                format!("{} has no profile yet.", user.name)
            };
            ctx.say(msg).await?;
        }
        Some(p) => {
            let embed = serenity::CreateEmbed::new()
                .author(serenity::CreateEmbedAuthor::new(user.name.clone()).icon_url(user.face()))
                .colour(serenity::Colour::from_rgb(245, 158, 11))
                .field("💰 Coins", p.balance.to_string(), true)
                .field("⭐ XP", p.xp.to_string(), true)
                .field("🏆 Level", p.level.to_string(), true);

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }
    Ok(())
}
