use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// Returns true if the given string is a plausible https/http URL.
/// We deliberately do not enforce file extensions because many CDN URLs
/// (e.g. Discord attachment URLs) lack them yet are valid images.
fn is_valid_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    (url_lower.starts_with("https://") || url_lower.starts_with("http://"))
        && url_lower.len() > 8 // at least one char after the scheme
}

/// Send a custom embed message
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    required_permissions = "MANAGE_MESSAGES"
)]
pub async fn embed(
    ctx: Context<'_>,
    #[description = "Embed title (max 256 chars)"] title: String,
    #[description = "Embed description (max 4096 chars)"] description: String,
    #[description = "Hex color (e.g. #ff0000)"] color: Option<String>,
    #[description = "Footer text (max 2048 chars)"] footer: Option<String>,
    #[description = "Image URL"] image: Option<String>,
    #[description = "Thumbnail URL"] thumbnail: Option<String>,
) -> Result<(), Error> {
    // Enforce Discord embed field length limits up-front
    if title.len() > 256 {
        ctx.say("❌ Title must be 256 characters or fewer.").await?;
        return Ok(());
    }
    if description.len() > 4096 {
        ctx.say("❌ Description must be 4096 characters or fewer.").await?;
        return Ok(());
    }
    if let Some(f) = &footer {
        if f.len() > 2048 {
            ctx.say("❌ Footer must be 2048 characters or fewer.").await?;
            return Ok(());
        }
    }

    let hex = color.as_deref().unwrap_or("#5865f2");
    let colour = u32::from_str_radix(hex.trim_start_matches('#'), 16)
        .map(serenity::Colour)
        .unwrap_or(serenity::Colour::from_rgb(88, 101, 242));

    let mut embed = serenity::CreateEmbed::new()
        .title(&title)
        .description(&description)
        .colour(colour);

    if let Some(f) = footer {
        embed = embed.footer(serenity::CreateEmbedFooter::new(f));
    }

    if let Some(img) = &image {
        if !is_valid_url(img) {
            ctx.say("❌ Image must be a valid https:// or http:// URL.").await?;
            return Ok(());
        }
        embed = embed.image(img.clone());
    }

    if let Some(thumb) = &thumbnail {
        if !is_valid_url(thumb) {
            ctx.say("❌ Thumbnail must be a valid https:// or http:// URL.").await?;
            return Ok(());
        }
        embed = embed.thumbnail(thumb.clone());
    }

    ctx.channel_id()
        .send_message(ctx, serenity::CreateMessage::new().embed(embed))
        .await?;

    ctx.say("✅ Embed sent.").await?;
    Ok(())
}
