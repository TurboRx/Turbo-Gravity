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
        .author(
            serenity::CreateEmbedAuthor::new(user.name.clone()).icon_url(user.face()),
        )
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

        let is_booster = m.premium_since.is_some();
        embed = embed.field("Server Booster", if is_booster { "Yes" } else { "No" }, true);

        let mut roles: Vec<(i64, serenity::RoleId)> = m
            .roles(ctx)
            .map(|roles| {
                roles
                    .iter()
                    .filter(|r| {
                        ctx.guild_id()
                            .map(|gid| r.id != gid.everyone_role())
                            .unwrap_or(true)
                    })
                    .map(|r| (r.position, r.id))
                    .collect()
            })
            .unwrap_or_default();

        roles.sort_by(|a, b| b.0.cmp(&a.0));

        const MAX_ROLES_SHOWN: usize = 10;
        let total_roles = roles.len();
        let shown: Vec<String> = roles
            .iter()
            .take(MAX_ROLES_SHOWN)
            .map(|(_, id)| format!("<@&{id}>"))
            .collect();

        let roles_str = if shown.is_empty() {
            "None".to_string()
        } else if total_roles > MAX_ROLES_SHOWN {
            format!("{} (+{} more)", shown.join(", "), total_roles - MAX_ROLES_SHOWN)
        } else {
            shown.join(", ")
        };
        embed = embed.field(format!("Roles [{total_roles}]"), roles_str, false);

        let perms = m
            .permissions(ctx)
            .unwrap_or(serenity::Permissions::empty());

        let mut key_perms: Vec<&str> = Vec::new();
        if perms.administrator() {
            key_perms.push("Administrator");
        } else {
            if perms.manage_guild() { key_perms.push("Manage Server"); }
            if perms.manage_channels() { key_perms.push("Manage Channels"); }
            if perms.manage_roles() { key_perms.push("Manage Roles"); }
            if perms.manage_messages() { key_perms.push("Manage Messages"); }
            if perms.kick_members() { key_perms.push("Kick Members"); }
            if perms.ban_members() { key_perms.push("Ban Members"); }
            if perms.moderate_members() { key_perms.push("Timeout Members"); }
            if perms.mention_everyone() { key_perms.push("Mention Everyone"); }
        }

        let perms_str = if key_perms.is_empty() {
            "None".to_string()
        } else {
            key_perms.join(", ")
        };
        embed = embed.field("Key Permissions", perms_str, false);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
