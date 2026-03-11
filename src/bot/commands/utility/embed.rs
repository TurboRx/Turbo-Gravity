use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

fn is_valid_image_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    (url_lower.starts_with("http://") || url_lower.starts_with("https://"))
        && (url_lower.contains(".png")
            || url_lower.contains(".jpg")
            || url_lower.contains(".jpeg")
            || url_lower.contains(".gif")
            || url_lower.contains(".webp"))
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
    #[description = "Embed title"] title: String,
    #[description = "Embed description"] description: String,
    #[description = "Hex color (e.g. #ff0000)"] color: Option<String>,
    #[description = "Footer text"] footer: Option<String>,
    #[description = "Image URL"] image: Option<String>,
    #[description = "Thumbnail URL"] thumbnail: Option<String>,
) -> Result<(), Error> {
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
        if !is_valid_image_url(img) {
            ctx.say("Invalid image URL. Must be a direct link ending in .png, .jpg, .gif, or .webp.")
                .await?;
            return Ok(());
        }
        embed = embed.image(img.clone());
    }

    if let Some(thumb) = &thumbnail {
        if !is_valid_image_url(thumb) {
            ctx.say("Invalid thumbnail URL. Must be a direct link ending in .png, .jpg, .gif, or .webp.")
                .await?;
            return Ok(());
        }
        embed = embed.thumbnail(thumb.clone());
    }

    ctx.channel_id()
        .send_message(ctx, serenity::CreateMessage::new().embed(embed))
        .await?;

    ctx.say("Embed sent.").await?;
    Ok(())
}
