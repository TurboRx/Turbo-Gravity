use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Get the avatar of a user
#[poise::command(slash_command, ephemeral)]
pub async fn avatar(
    ctx: Context<'_>,
    #[description = "User to view"] target: Option<serenity::User>,
) -> Result<(), Error> {
    let user = target.as_ref().unwrap_or_else(|| ctx.author());
    let url = user.face();
    ctx.say(format!("{}'s avatar: {url}", user.name)).await?;
    Ok(())
}
