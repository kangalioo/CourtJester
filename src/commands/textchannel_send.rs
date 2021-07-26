use rand::prelude::*;
use serenity::{framework::standard::CommandResult, model::prelude::*};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;

use crate::helpers::{command_utils, permissions_helper};

struct TextChannels {
    nice_id: Option<i64>,
    bruh_id: Option<i64>,
    quote_id: Option<i64>,
}

/// If prefix: gets invoking message link
/// If slash: sends a response and retrieves its link
/// When send_in_any_case true, the response is sent in any case
async fn get_or_send_message_url(
    ctx: crate::Context<'_>,
    guild_id: GuildId,
    response: &str,
    send_in_any_case: bool,
) -> CommandResult<String> {
    Ok(match ctx {
        crate::Context::Prefix(ctx) => {
            if send_in_any_case {
                poise::say_prefix_reply(ctx, response.into()).await?;
            }
            command_utils::get_message_url(guild_id, ctx.msg.channel_id, ctx.msg.id)
        }
        crate::Context::Slash(ctx) => {
            // Sending a message is crucial for two reasons:
            // - the interaction is shown as failed if we don't send a response
            // - we need a message to link to
            poise::say_slash_reply(ctx, response.into()).await?;
            ctx.interaction
                .channel_id
                .messages(ctx.discord, |f| f.limit(1))
                .await?[0]
                .link_ensured(ctx.discord)
                .await
        }
    })
}

/// Sends `nice` to a specified channel. Provide a channel as the first argument to set it
///
/// Usage: `nice <message>` or `nice <channel>`
#[poise::command(slash_command)]
pub async fn nice(
    ctx: crate::Context<'_>,
    #[description = "Specify a target channel for future invocations of this command (moderators only)"]
    channel: Option<ChannelId>,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();

    let pool = &ctx.data().connection_pool;

    let check = sqlx::query!(
        "SELECT EXISTS(SELECT nice_id FROM text_channels WHERE guild_id = $1)",
        guild_id.0 as i64
    )
    .fetch_one(pool)
    .await?;

    if let Some(channel_id) = channel {
        if permissions_helper::check_permission_2(ctx, None, false).await? {
            if check.exists.unwrap() {
                sqlx::query!(
                    "UPDATE text_channels SET nice_id = $1 WHERE guild_id = $2",
                    channel_id.0 as i64,
                    guild_id.0 as i64
                )
                .execute(pool)
                .await?;
            } else {
                insert_or_update(pool, guild_id, "nice", channel_id.0 as i64).await?;
            }

            poise::say_reply(ctx, "Channel sucessfully set!".into()).await?;
        }

        return Ok(());
    }

    if !check.exists.unwrap() {
        poise::say_reply(
            ctx,
            "The Nice channel isn't set! Please specify a channel!".into(),
        )
        .await?;
        return Ok(());
    }

    let channel_num = get_channels(pool, guild_id).await?;

    if channel_num.nice_id.is_none() {
        poise::say_reply(
            ctx,
            "The Nice channel isn't set! Please specify a channel!".into(),
        )
        .await?;
        return Ok(());
    }

    let message_url = get_or_send_message_url(ctx, guild_id, "Nice indeed", false).await?;

    ChannelId(channel_num.nice_id.unwrap() as u64)
        .send_message(ctx.discord(), |m| {
            m.embed(|e| {
                e.color(0x290e05);
                e.title(format!("Nice - {}", ctx.author().name));
                e.field("Source", format!("[Jump!]({})", message_url), false)
            })
        })
        .await?;

    Ok(())
}

/// Sends `bruh` to a specified channel. Provide a channel as the first argument to set it
///
/// Usage: `bruh <message>` or `bruh <channel>`
#[poise::command(slash_command)]
pub async fn bruh(
    ctx: crate::Context<'_>,
    #[description = "Specify a target channel for future invocations of this command (moderators only)"]
    channel: Option<ChannelId>,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();

    let pool = &ctx.data().connection_pool;

    let check = sqlx::query!(
        "SELECT EXISTS(SELECT bruh_id FROM text_channels WHERE guild_id = $1)",
        guild_id.0 as i64
    )
    .fetch_one(pool)
    .await?;

    if let Some(channel_id) = channel {
        if permissions_helper::check_permission_2(ctx, None, false).await? {
            if check.exists.unwrap() {
                sqlx::query!(
                    "UPDATE text_channels SET bruh_id = $1 WHERE guild_id = $2",
                    channel_id.0 as i64,
                    guild_id.0 as i64
                )
                .execute(pool)
                .await?;
            } else {
                insert_or_update(pool, guild_id, "bruh", channel_id.0 as i64).await?;
            }

            poise::say_reply(ctx, "Channel sucessfully set!".into()).await?;
        }

        return Ok(());
    }

    if !check.exists.unwrap() {
        poise::say_reply(
            ctx,
            "The Bruh channel isn't set! Please specify a channel!".into(),
        )
        .await?;
        return Ok(());
    }

    let channel_nums = get_channels(pool, guild_id).await?;

    if channel_nums.bruh_id.is_none() {
        poise::say_reply(
            ctx,
            "The Bruh channel isn't set! Please specify a channel!".into(),
        )
        .await?;
        return Ok(());
    }

    let message_url = get_or_send_message_url(ctx, guild_id, "***BRUH MOMENT***", true).await?;

    ChannelId(channel_nums.bruh_id.unwrap() as u64)
        .send_message(ctx.discord(), |m| {
            m.embed(|e| {
                e.color(0xfc5e03);
                e.title("Ladies and Gentlemen!");
                e.description(format!(
                    "A bruh moment has been declared by {}",
                    ctx.author().mention()
                ));
                e.field("Source", format!("[Jump!]({})", message_url), false)
            })
        })
        .await?;

    Ok(())
}

/// Quotes yourself or a specified user
///
/// Usage: `quote <user mention> <content>` or `quote <content>`
#[poise::command(slash_command)]
pub async fn quote(
    ctx: crate::Context<'_>,
    // TODO: make these work properly
    #[description = "Who said the quote"] quoted_user: Option<Member>,
    #[description = "Specify a target channel for future invocations of this command (moderators only)"]
    channel: Option<ChannelId>,
    #[rest]
    #[description = "Content of the quote"]
    quote_content: Option<String>,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();

    let pool = &ctx.data().connection_pool;

    let starboard_data = sqlx::query!(
        "SELECT starboard_threshold FROM guild_info WHERE guild_id = $1",
        guild_id.0 as i64
    )
    .fetch_one(pool)
    .await?;

    if starboard_data.starboard_threshold.is_some() {
        poise::say_reply(
            ctx,
            "You can't use the quote command because starboard is enabled in this server!".into(),
        )
        .await?;
        return Ok(());
    }

    let check = sqlx::query!(
        "SELECT EXISTS(SELECT quote_id FROM text_channels WHERE guild_id = $1)",
        guild_id.0 as i64
    )
    .fetch_one(pool)
    .await?;

    if let Some(channel_id) = channel {
        if permissions_helper::check_permission_2(ctx, None, false).await? {
            if check.exists.unwrap() {
                sqlx::query!(
                    "UPDATE text_channels SET quote_id = $1 WHERE guild_id = $2",
                    channel_id.0 as i64,
                    guild_id.0 as i64
                )
                .execute(pool)
                .await?;
            } else {
                insert_or_update(pool, guild_id, "quote", channel_id.0 as i64).await?;
            }

            poise::say_reply(ctx, "Channel sucessfully set!".into()).await?;
        }

        return Ok(());
    }

    if !check.exists.unwrap() {
        poise::say_reply(
            ctx,
            "The Quote channel isn't set! Please specify a channel!".into(),
        )
        .await?;
        return Ok(());
    }

    let channels = get_channels(pool, guild_id).await?;

    if channels.quote_id.is_none() {
        poise::say_reply(
            ctx,
            "The Quote channel isn't set! Please specify a channel!".into(),
        )
        .await?;
        return Ok(());
    }

    let message_url = get_or_send_message_url(ctx, guild_id, "Successfully quoted", false).await?;

    let user = match &quoted_user {
        Some(user) => &user.user,
        None => ctx.author(),
    };

    let avatar_id = user
        .avatar_url()
        .unwrap_or_else(|| user.default_avatar_url());

    ChannelId(channels.quote_id.unwrap() as u64)
        .send_message(ctx.discord(), |m| {
            m.embed(|e| {
                e.color(0xfabe21);
                e.author(|a| {
                    a.name(&user.name);
                    a.icon_url(&avatar_id);
                    a
                });
                e.description(quote_content.unwrap_or_default());
                e.field("Source", format!("[Jump!]({})", message_url), false)
            })
        })
        .await?;

    Ok(())
}

/// Will you pass the vibecheck?
#[poise::command(slash_command, discard_spare_arguments)]
pub async fn vibecheck(ctx: crate::Context<'_>) -> CommandResult {
    poise::say_reply(ctx, "Initiating vibe check...".into()).await?;

    sleep(Duration::from_secs(3)).await;

    if random() {
        let success_vec = vec![
            "Continue vibing good sir/madam",
            "Have a wonderful day",
            "Your wish will come true",
            "STRAIGHT vibing! I like that",
            "Drop your favorite vibes in the chat",
        ];

        let mut rng = StdRng::from_entropy();

        let val = rng.gen_range(0..=success_vec.len() - 1);

        poise::say_reply(
            ctx,
            format!(
                "{} has passed the vibe check. {}.",
                ctx.author().mention(),
                success_vec[val]
            ),
        )
        .await?;
    } else {
        poise::say_reply(
            ctx,
            format!(
                "{} has failed the vibe check. Show me your vibing license!",
                ctx.author().mention()
            ),
        )
        .await?;
    }
    Ok(())
}

async fn get_channels(
    pool: &PgPool,
    guild_id: GuildId,
) -> Result<TextChannels, Box<dyn std::error::Error + Send + Sync>> {
    let data = sqlx::query_as!(
        TextChannels,
        "SELECT nice_id, bruh_id, quote_id FROM text_channels WHERE guild_id = $1",
        guild_id.0 as i64
    )
    .fetch_one(pool)
    .await?;

    Ok(data)
}

async fn insert_or_update(
    pool: &PgPool,
    guild_id: GuildId,
    channel_type: &str,
    channel_id: i64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match channel_type {
        "nice" => {
            sqlx::query!(
                "INSERT INTO text_channels VALUES($1, $2, null, null)
                        ON CONFLICT (guild_id)
                        DO UPDATE SET nice_id = $2",
                guild_id.0 as i64,
                channel_id
            )
            .execute(pool)
            .await?;
        }
        "bruh" => {
            sqlx::query!(
                "INSERT INTO text_channels VALUES($1, null, $2, null)
                        ON CONFLICT (guild_id)
                        DO UPDATE SET bruh_id = $2",
                guild_id.0 as i64,
                channel_id
            )
            .execute(pool)
            .await?;
        }
        "quote" => {
            sqlx::query!(
                "INSERT INTO text_channels VALUES($1, null, null, $2)
                        ON CONFLICT (guild_id)
                        DO UPDATE SET quote_id = $2",
                guild_id.0 as i64,
                channel_id
            )
            .execute(pool)
            .await?;
        }
        _ => {}
    }

    Ok(())
}

pub async fn sender_help(ctx: crate::Context<'_>) {
    let content = concat!(
        "nice: Sends nice to a defined channel \n\n",
        "bruh: Sends a bruh moment to a defined channel \n\n",
        "quote <author> <text>: Quotes a user. Deactivated when starboard is enabled \n\n",
        "vibecheck: Checks your vibe. Try it out!"
    );

    let _ = poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("Textchannel Sender Help");
            e.description("Description: Commands that send messages to specified channels");
            e.field("Commands", content, false);
            e.footer(|f| {
                f.text("Adding a channel mention will set the sender channel (Moderator only)");
                f
            });
            e
        })
    })
    .await;
}
