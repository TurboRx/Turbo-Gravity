// Member::permissions is deprecated in serenity 0.12 in favour of
// Guild::user_permissions_in which considers overwrites.  For our simple
// MANAGE_CHANNELS check the base-guild permissions are sufficient, and
// holding a Guild CacheRef across async boundaries is not allowed.
#![allow(deprecated)]

use crate::bot::{Context, Error};
use poise::serenity_prelude as serenity;

const TOPIC_PREFIX: &str = "Ticket owner:";

fn extract_owner_id(topic: &str) -> Option<&str> {
    topic
        .strip_prefix(TOPIC_PREFIX)
        .map(str::trim)
        .and_then(|s| s.split('|').next())
        .map(str::trim)
}

// ---------------------------------------------------------------------------
// Subcommands
// ---------------------------------------------------------------------------

/// Create a private support ticket
#[poise::command(slash_command, guild_only, ephemeral)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Reason for the ticket"] reason: Option<String>,
    #[description = "Category to place the ticket in"]
    #[channel_types("Category")]
    category: Option<serenity::GuildChannel>,
    #[description = "Staff role to notify and grant access"] staff_role: Option<serenity::Role>,
) -> Result<(), Error> {
    let reason = reason
        .as_deref()
        .unwrap_or("No reason provided")
        .to_string();

    // Extract everything needed from the non-Send CacheRef before any await
    let (guild_id, everyone_role, bot_id, existing_ticket) = {
        let guild = ctx
            .guild()
            .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
        let author_id_str = ctx.author().id.to_string();
        let existing = guild
            .channels
            .values()
            .find(|ch| {
                ch.topic
                    .as_deref()
                    .and_then(|t| extract_owner_id(t))
                    .map(str::trim)
                    == Some(author_id_str.as_str())
            })
            .map(|ch| ch.id);
        (
            guild.id,
            guild.id.everyone_role(),
            ctx.cache().current_user().id,
            existing,
        )
    };

    if let Some(existing_id) = existing_ticket {
        ctx.say(format!("You already have an open ticket: <#{existing_id}>"))
            .await?;
        return Ok(());
    }

    let username = ctx.author().name.to_lowercase();
    let safe_name: String = username.chars().filter(|c| c.is_alphanumeric()).collect();
    let disc = ctx
        .author()
        .discriminator
        .map(|d| d.get().to_string())
        .unwrap_or_else(|| {
            let id = ctx.author().id.to_string();
            id.chars()
                .rev()
                .take(4)
                .collect::<String>()
                .chars()
                .rev()
                .collect()
        });
    let channel_name = format!("ticket-{safe_name}-{disc}")
        .chars()
        .take(90)
        .collect::<String>();
    let topic = format!("{TOPIC_PREFIX} {} | Reason: {reason}", ctx.author().id);

    let mut overwrites = vec![
        serenity::PermissionOverwrite {
            allow: serenity::Permissions::empty(),
            deny: serenity::Permissions::VIEW_CHANNEL | serenity::Permissions::SEND_MESSAGES,
            kind: serenity::PermissionOverwriteType::Role(everyone_role),
        },
        serenity::PermissionOverwrite {
            allow: serenity::Permissions::VIEW_CHANNEL
                | serenity::Permissions::SEND_MESSAGES
                | serenity::Permissions::READ_MESSAGE_HISTORY,
            deny: serenity::Permissions::empty(),
            kind: serenity::PermissionOverwriteType::Member(ctx.author().id),
        },
        serenity::PermissionOverwrite {
            allow: serenity::Permissions::VIEW_CHANNEL
                | serenity::Permissions::SEND_MESSAGES
                | serenity::Permissions::MANAGE_CHANNELS,
            deny: serenity::Permissions::empty(),
            kind: serenity::PermissionOverwriteType::Member(bot_id),
        },
    ];

    if let Some(ref role) = staff_role {
        overwrites.push(serenity::PermissionOverwrite {
            allow: serenity::Permissions::VIEW_CHANNEL
                | serenity::Permissions::SEND_MESSAGES
                | serenity::Permissions::READ_MESSAGE_HISTORY,
            deny: serenity::Permissions::empty(),
            kind: serenity::PermissionOverwriteType::Role(role.id),
        });
    }

    let mut builder = serenity::CreateChannel::new(channel_name)
        .kind(serenity::ChannelType::Text)
        .topic(&topic)
        .permissions(overwrites);

    if let Some(ref cat) = category {
        if cat.kind == serenity::ChannelType::Category {
            builder = builder.category(cat.id);
        }
    }

    let channel = guild_id.create_channel(ctx, builder).await?;

    let embed = serenity::CreateEmbed::new()
        .title("Support Ticket")
        .description("Thank you for reaching out. A team member will assist you shortly.")
        .colour(serenity::Colour::from_rgb(14, 165, 233))
        .field("Opened by", format!("<@{}>", ctx.author().id), true)
        .field("Reason", &reason, true);

    let mut msg = serenity::CreateMessage::new().embed(embed);
    if let Some(ref role) = staff_role {
        msg = msg.content(format!("<@&{}>", role.id));
    }
    channel.id.send_message(ctx, msg).await?;

    ctx.say(format!("Ticket created: <#{}>", channel.id))
        .await?;
    Ok(())
}

/// Close the current ticket
#[poise::command(slash_command, guild_only, ephemeral)]
pub async fn close(
    ctx: Context<'_>,
    #[description = "Close reason"] reason: Option<String>,
    #[description = "Delete after X minutes (1-1440)"]
    #[min = 1_u32]
    #[max = 1440_u32]
    delete_after: Option<u32>,
) -> Result<(), Error> {
    let reason = reason
        .as_deref()
        .unwrap_or("No reason provided")
        .to_string();

    // Extract @everyone RoleId before any await (CacheRef is !Send)
    let everyone_id = {
        let guild = ctx
            .guild()
            .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
        guild.id.everyone_role()
    };

    let channel = ctx
        .channel_id()
        .to_channel(ctx)
        .await?
        .guild()
        .ok_or_else(|| anyhow::anyhow!("Not a guild channel"))?;

    let owner_id = channel
        .topic
        .as_deref()
        .and_then(|t| extract_owner_id(t).map(str::to_string));

    if owner_id.is_none() {
        ctx.say("This does not appear to be a ticket channel.")
            .await?;
        return Ok(());
    }

    let is_owner = owner_id.as_deref() == Some(&ctx.author().id.to_string());
    let invoker = ctx.author_member().await;
    let is_mod = invoker
        .as_ref()
        .is_some_and(|m| {
            m.permissions(ctx)
                .map(|p| p.contains(serenity::Permissions::MANAGE_CHANNELS))
                .unwrap_or(false)
        });

    if !is_owner && !is_mod {
        ctx.say("You cannot close this ticket.").await?;
        return Ok(());
    }

    ctx.channel_id()
        .create_permission(
            ctx,
            serenity::PermissionOverwrite {
                allow: serenity::Permissions::empty(),
                deny: serenity::Permissions::SEND_MESSAGES,
                kind: serenity::PermissionOverwriteType::Role(everyone_id),
            },
        )
        .await?;

    ctx.say(format!(
        "🔒 Ticket closed. Reason: {reason}{}",
        delete_after
            .map(|d| format!(" | Deleting in {d} minute(s)"))
            .unwrap_or_default()
    ))
    .await?;

    if let Some(mins) = delete_after {
        let channel_id = ctx.channel_id();
        let http = ctx.serenity_context().http.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(u64::from(mins) * 60)).await;
            let _ = channel_id.delete(&http).await;
        });
    }

    Ok(())
}

/// Add a user to this ticket
#[poise::command(slash_command, guild_only, ephemeral)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "User to add"] user: serenity::User,
) -> Result<(), Error> {
    let channel = ctx
        .channel_id()
        .to_channel(ctx)
        .await?
        .guild()
        .ok_or_else(|| anyhow::anyhow!("Not a guild channel"))?;

    let owner_id = channel
        .topic
        .as_deref()
        .and_then(|t| extract_owner_id(t).map(str::to_string));

    if owner_id.is_none() {
        ctx.say("This does not appear to be a ticket channel.")
            .await?;
        return Ok(());
    }

    let is_owner = owner_id.as_deref() == Some(&ctx.author().id.to_string());
    let invoker = ctx.author_member().await;
    let is_mod = invoker
        .as_ref()
        .is_some_and(|m| {
            m.permissions(ctx)
                .map(|p| p.contains(serenity::Permissions::MANAGE_CHANNELS))
                .unwrap_or(false)
        });

    if !is_owner && !is_mod {
        ctx.say("You cannot modify this ticket.").await?;
        return Ok(());
    }

    if owner_id.as_deref() == Some(&user.id.to_string()) {
        ctx.say("That user is already the ticket owner.").await?;
        return Ok(());
    }

    ctx.channel_id()
        .create_permission(
            ctx,
            serenity::PermissionOverwrite {
                allow: serenity::Permissions::VIEW_CHANNEL
                    | serenity::Permissions::SEND_MESSAGES
                    | serenity::Permissions::READ_MESSAGE_HISTORY,
                deny: serenity::Permissions::empty(),
                kind: serenity::PermissionOverwriteType::Member(user.id),
            },
        )
        .await?;

    ctx.say(format!("✅ Added {} to the ticket.", user.name))
        .await?;
    Ok(())
}

/// Remove a user from this ticket
#[poise::command(slash_command, guild_only, ephemeral)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "User to remove"] user: serenity::User,
) -> Result<(), Error> {
    let channel = ctx
        .channel_id()
        .to_channel(ctx)
        .await?
        .guild()
        .ok_or_else(|| anyhow::anyhow!("Not a guild channel"))?;

    let owner_id = channel
        .topic
        .as_deref()
        .and_then(|t| extract_owner_id(t).map(str::to_string));

    if owner_id.is_none() {
        ctx.say("This does not appear to be a ticket channel.")
            .await?;
        return Ok(());
    }

    let is_owner = owner_id.as_deref() == Some(&ctx.author().id.to_string());
    let invoker = ctx.author_member().await;
    let is_mod = invoker
        .as_ref()
        .is_some_and(|m| {
            m.permissions(ctx)
                .map(|p| p.contains(serenity::Permissions::MANAGE_CHANNELS))
                .unwrap_or(false)
        });

    if !is_owner && !is_mod {
        ctx.say("You cannot modify this ticket.").await?;
        return Ok(());
    }

    if owner_id.as_deref() == Some(&user.id.to_string()) {
        ctx.say("Cannot remove the ticket owner.").await?;
        return Ok(());
    }

    if user.id == ctx.cache().current_user().id {
        ctx.say("Cannot remove the bot from the ticket.").await?;
        return Ok(());
    }

    ctx.channel_id()
        .delete_permission(ctx, serenity::PermissionOverwriteType::Member(user.id))
        .await?;

    ctx.say(format!("❌ Removed {} from the ticket.", user.name))
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Parent command
// ---------------------------------------------------------------------------

/// Advanced ticket controls
#[poise::command(
    slash_command,
    guild_only,
    subcommands("create", "close", "add", "remove")
)]
pub async fn ticket(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
