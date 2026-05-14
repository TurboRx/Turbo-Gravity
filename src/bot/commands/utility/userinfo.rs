use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

/// View info about a user
#[poise::command(slash_command, ephemeral)]
pub async fn userinfo(
    ctx: Context<'_>,
    #[description = "User to look up"] target: Option<serenity::User>,
) -> Result<(), Error> {
    let user = target.as_ref().unwrap_or_else(|| ctx.author());

    // Extract member info and colour from CacheRef before any await
    let (member_opt, colour) = {
        let guild = ctx.guild();
        let member = guild
            .as_ref()
            .and_then(|g| g.members.get(&user.id).cloned());
        let colour = member
            .as_ref()
            .and_then(|m: &serenity::Member| m.colour(ctx))
            .unwrap_or(serenity::Colour::from_rgb(88, 101, 242));
        (member, colour)
    };

    let created_ts = user.id.created_at().unix_timestamp();

    let mut embed = serenity::CreateEmbed::new()
        .author(serenity::CreateEmbedAuthor::new(user.name.clone()).icon_url(user.face()))
        .thumbnail(user.face())
        .colour(colour)
        .field("User ID", user.id.to_string(), true)
        .field("Bot", if user.bot { "Yes" } else { "No" }, true)
        .field("Account Created", format!("<t:{created_ts}:R>"), true);

    if let Some(m) = &member_opt {
        if let Some(joined) = m.joined_at {
            embed = embed.field(
                "Joined Server",
                format!("<t:{}:R>", joined.unix_timestamp()),
                true,
            );
        }
        let roles: Vec<String> = m
            .roles(ctx)
            .map(|roles| {
                roles
                    .iter()
                    .filter(|r| {
                        ctx.guild_id()
                            .is_none_or(|gid| r.id != gid.everyone_role())
                    })
                    .map(|r| format!("<@&{}>", r.id))
                    .collect()
            })
            .unwrap_or_default();

        let roles_str = if roles.is_empty() {
            "None".to_string()
        } else {
            roles.join(", ")
        };
        embed = embed.field("Roles", roles_str, false);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
