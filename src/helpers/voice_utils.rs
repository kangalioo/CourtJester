use futures::future::{AbortHandle, Abortable};
use serenity::{
    client::Context,
    framework::standard::CommandResult,
    model::{
        guild::Guild,
        id::{ChannelId, GuildId},
    },
};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

pub async fn get_voice_state(ctx: crate::Context<'_>, guild: &Guild) -> CommandResult<bool> {
    if !(guild.voice_states.contains_key(&ctx.author().id)
        || guild.voice_states.contains_key(&ctx.data().bot_id))
    {
        return Ok(false);
    }

    let user_voice_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|state| state.channel_id);
    let bot_voice_id = guild
        .voice_states
        .get(&ctx.data().bot_id)
        .and_then(|state| state.channel_id);

    if user_voice_id == bot_voice_id {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Forces the bot to join the voice chat
#[poise::command(slash_command, aliases("connect"))]
pub async fn summon(ctx: crate::Context<'_>) -> CommandResult {
    let guild = ctx.guild().await.unwrap();

    if guild.voice_states.contains_key(&ctx.data().bot_id) {
        poise::say_reply(ctx, "Looks like I'm already in a voice channel! Please disconnect me before summoning me again!".into())
            .await?;

        return Ok(());
    }

    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let voice_channel = match channel_id {
        Some(channel) => channel,
        None => {
            poise::say_reply(ctx, "Please join a voice channel!".into()).await?;

            return Ok(());
        }
    };

    match join_voice_internal(ctx, voice_channel).await {
        Ok(_) => {
            poise::say_reply(
                ctx,
                format!(
                    "Joined {}",
                    voice_channel.name(ctx.discord()).await.unwrap()
                ),
            )
            .await?;

            let ctx_clone = ctx.discord().clone();
            let data = Arc::clone(&ctx.data());
            tokio::spawn(async move {
                create_new_timer(ctx_clone, guild.id, data).await;
            });
        }
        Err(_e) => {
            poise::say_reply(ctx, "I couldn't join the voice channel. Please check if I have permission to access it!".into())
                .await?;
        }
    }

    Ok(())
}

pub async fn join_voice_internal(
    ctx: crate::Context<'_>,
    voice_channel: ChannelId,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.discord()).await.unwrap().clone();

    let (_, handler) = manager.join_gateway(guild_id, voice_channel).await;

    match handler {
        Ok(conn_info) => ctx.data().lavalink.create_session(&conn_info).await?,
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

/// Leaves the voice chat and clears everything
#[poise::command(slash_command, aliases("dc"))]
pub async fn disconnect(ctx: crate::Context<'_>) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();
    let guild = ctx.guild().await.unwrap();

    if !get_voice_state(ctx, &guild).await? {
        poise::say_reply(
            ctx,
            "Please be in a voice channel or in the same voice channel as me!".into(),
        )
        .await?;
        return Ok(());
    }

    match leavevc_internal(&ctx.discord(), guild_id, Arc::clone(ctx.data())).await {
        Ok(_) => {
            let voice_timer_map = &ctx.data().voice_timer_map;
            if voice_timer_map.contains_key(&guild_id) {
                if let Some(future_guard) = voice_timer_map.get(&guild_id) {
                    future_guard.value().abort();
                }
                voice_timer_map.remove(&guild_id);
            }

            poise::say_reply(ctx, "Left the voice channel!".into()).await?;
        }
        Err(_e) => {
            poise::say_reply(ctx, "The bot isn't in a voice channel!".into()).await?;
        }
    }

    Ok(())
}

pub async fn leavevc_internal(
    ctx: &Context,
    guild_id: GuildId,
    data: Arc<crate::Data>,
) -> CommandResult {
    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        manager.remove(guild_id).await?;

        data.lavalink.destroy(guild_id).await?;

        {
            let nodes = data.lavalink.nodes().await;
            nodes.remove(&guild_id.0);

            let loops = data.lavalink.loops().await;
            loops.remove(&guild_id.0);
        }
    } else {
        return Err("The bot isn't in a voice channel!".into());
    }

    Ok(())
}

pub async fn create_new_timer(ctx: Context, guild_id: GuildId, data: Arc<crate::Data>) {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(
        leavevc_internal(&ctx, guild_id, Arc::clone(&data)),
        abort_registration,
    );

    data.voice_timer_map.insert(guild_id, abort_handle);

    sleep(Duration::from_secs(300)).await;
    let _ = future.await;

    data.voice_timer_map.remove(&guild_id);
}

pub async fn voice_help(ctx: &Context, channel_id: ChannelId) {
    let content = concat!(
        "summon: Forces the bot to join the voice chat \nAlias: connect \n\n",
        "disconnect: Leaves the voice chat and clears everything \n\n"
    );

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Voice Help");
                e.description("Description: General commands for voice chat");
                e.field("Commands", content, false);
                e.footer(|f| {
                    f.text("The user has to be in the voice chat on execution!");
                    f
                });
                e
            })
        })
        .await;
}
