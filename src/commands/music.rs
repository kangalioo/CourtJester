use lavalink_rs::LavalinkClient;
use rust_clock::Clock;
use serenity::{
    builder::CreateEmbed,
    client::Context,
    framework::standard::CommandResult,
    model::{channel::ReactionType, id::GuildId},
};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

use crate::{
    helpers::{
        command_utils, permissions_helper,
        voice_utils::{self, get_voice_state},
    },
    JesterError, PermissionType,
};

async fn react(ctx: crate::Context<'_>, emoji: &str) -> CommandResult {
    match ctx {
        crate::Context::Prefix(ctx) => {
            ctx.msg
                .react(ctx.discord, ReactionType::Unicode(String::from(emoji)))
                .await?;
        }
        crate::Context::Slash(ctx) => poise::say_slash_reply(ctx, emoji.into()).await?,
    }
    Ok(())
}

/// Plays the specified track
#[poise::command(slash_command, aliases("p"))]
pub async fn play(
    ctx: crate::Context<'_>,
    #[rest]
    #[description = "URL or search keywords"]
    query: String,
) -> CommandResult {
    let guild = ctx.guild().await.unwrap();
    let guild_id = ctx.guild_id().unwrap();

    let bot_id = &ctx.data().bot_id;

    // TODO: Doesn't auto-summon the bot if the bot isn't in the voice channel. Check if queue is empty before running
    if guild.voice_states.contains_key(&bot_id) && !get_voice_state(ctx, &guild).await? {
        poise::say_reply(
            ctx,
            "Please be in a voice channel or in the same voice channel as me!".into(),
        )
        .await?;
        return Ok(());
    }

    let voice_channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let voice_channel = voice_channel_id.unwrap();

    if query.is_empty() {
        poise::say_reply(ctx, "Please enter a track URL after the command!".into()).await?;
        return Ok(());
    }

    let manager = songbird::get(ctx.discord()).await.unwrap();
    let voice_timer_map = &ctx.data().voice_timer_map;

    if manager.get(guild_id).is_none() {
        voice_utils::join_voice_internal(ctx, voice_channel).await?;
    }

    if voice_timer_map.contains_key(&guild_id) {
        if let Some(future_guard) = voice_timer_map.get(&guild_id) {
            future_guard.value().abort();
        }
        voice_timer_map.remove(&guild_id);
    }

    let query = if query.contains("https://open.spotify.com") {
        let track_id = match query.rsplit('/').next() {
            Some(id) => id,
            None => {
                poise::say_reply(
                    ctx,
                    JesterError::MissingError("valid Spotify URL").to_string(),
                )
                .await?;
                return Ok(());
            }
        };

        match get_spotify_track_info(track_id, ctx).await {
            Some(track_info) => track_info,
            None => {
                poise::say_reply(
                    ctx,
                    "Couldn't find the track on spotify! Check the URL?".into(),
                )
                .await?;
                return Ok(());
            }
        }
    } else {
        query.to_string()
    };

    let lava_client = &ctx.data().lavalink;

    let query_info = lava_client.auto_search_tracks(&query).await?;

    if query_info.tracks.is_empty() {
        poise::say_reply(
            ctx,
            "Couldn't find the video on YouTube! Check the query?".into(),
        )
        .await?;
        return Ok(());
    }

    if let Err(e) = LavalinkClient::play(&lava_client, guild_id, query_info.tracks[0].clone())
        .queue()
        .await
    {
        return Err(e.into());
    };

    let track_info = query_info.tracks[0].info.as_ref();

    let mut cl = Clock::new();
    cl.set_time_ms(track_info.unwrap().length as i64);

    poise::send_reply(ctx, |m| {
        m.content("Added to queue:".into());
        m.embed(|e| {
            e.color(0x98fb98);
            e.title(&track_info.unwrap().title);
            e.url(&track_info.unwrap().uri);
            e.field("Uploader", &track_info.unwrap().author, true);
            e.field("Length", cl.get_time(), true);
            e.footer(|f| {
                f.text(format!("Requested by {}", ctx.author().name));
                f
            })
        })
    })
    .await?;

    let data = Arc::clone(ctx.data());
    let ctx = ctx.discord().clone();
    tokio::spawn(queue_checker(ctx, guild_id, data));

    Ok(())
}

pub async fn get_spotify_track_info(track_id: &str, ctx: crate::Context<'_>) -> Option<String> {
    let spotify = &ctx.data().spotify_client;

    if let Ok(track) = spotify.tracks().get_track(track_id, None).await {
        Some(track.data.name + " " + &track.data.artists[0].name)
    } else {
        None
    }
}

/// Pauses the current track
#[poise::command(slash_command)]
pub async fn pause(ctx: crate::Context<'_>) -> CommandResult {
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

    let lava_client = &ctx.data().lavalink;

    if lava_client.nodes().await.contains_key(&guild_id.0) {
        lava_client.pause(guild_id).await?;
        react(ctx, "⏸").await?;

        let ctx_clone = ctx.discord().clone();
        let data = Arc::clone(ctx.data());
        tokio::spawn(async move {
            voice_utils::create_new_timer(ctx_clone, guild_id, data).await;
        });
    };

    Ok(())
}

/// Stops the current track and empties the queue. Doesn't disconnect the bot
#[poise::command(slash_command)]
pub async fn stop(ctx: crate::Context<'_>) -> CommandResult {
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

    let lava_client = &ctx.data().lavalink;

    if !lava_client.nodes().await.contains_key(&guild_id.0) {
        poise::say_reply(
            ctx,
            "The bot isn't connected to a voice channel or node! Please re-run join or play!"
                .into(),
        )
        .await?;
        return Ok(());
    }

    lava_client.skip(guild_id).await;
    lava_client.stop(guild_id).await?;
    react(ctx, "🛑").await?;

    let ctx_clone = ctx.discord().clone();
    let data = Arc::clone(ctx.data());
    tokio::spawn(async move {
        voice_utils::create_new_timer(ctx_clone, guild_id, data).await;
    });

    Ok(())
}

/// Resumes the current track
#[poise::command(slash_command, aliases("unpause"))]
pub async fn resume(ctx: crate::Context<'_>) -> CommandResult {
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

    let lava_client = &ctx.data().lavalink;

    if !lava_client.nodes().await.contains_key(&guild_id.0) {
        poise::say_reply(
            ctx,
            "The bot isn't connected to a voice channel or node! Please re-run join or play!"
                .into(),
        )
        .await?;

        return Ok(());
    }

    let voice_timer_map = &ctx.data().voice_timer_map;

    if let Some(future_guard) = voice_timer_map.get(&guild_id) {
        future_guard.value().abort();
    }

    lava_client.resume(ctx.guild_id().unwrap()).await?;
    react(ctx, "▶").await?;

    Ok(())
}

/// See the current queue for the guild and what's playing
#[poise::command(slash_command, aliases("q"))]
pub async fn queue(ctx: crate::Context<'_>) -> CommandResult {
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

    let lava_client = &ctx.data().lavalink;

    let nodes = lava_client.nodes().await;
    let node = match nodes.get(&ctx.guild_id().unwrap().0) {
        Some(node) => node,
        None => {
            poise::say_reply(
                ctx,
                "The bot isn't connected to a voice channel or node! Please re-run join or play!"
                    .into(),
            )
            .await?;

            return Ok(());
        }
    };

    let queue = &node.queue;

    if queue.is_empty() && node.now_playing.is_none() {
        poise::say_reply(ctx, "The queue is currently empty!".into()).await?;

        return Ok(());
    }

    let mut eb = CreateEmbed::default();
    eb.color(0x0377fc);
    eb.title(format!(
        "Queue for {}",
        guild_id.name(ctx.discord()).await.unwrap()
    ));

    if let Some(t) = node.now_playing.as_ref() {
        let t_info = t.track.info.as_ref();

        let mut cl = Clock::new();
        cl.set_time_ms(t_info.unwrap().length as i64);
        eb.field(
            "Now Playing",
            format!(
                "[{}]({}) | `{}`",
                t_info.unwrap().title,
                t_info.unwrap().uri,
                cl.get_time()
            ),
            false,
        );
    }

    if queue.len() > 1 {
        let mut queue_string = String::new();

        for (num, t) in queue.iter().enumerate().skip(1) {
            let t_info = t.track.info.as_ref();

            let mut cl = Clock::new();
            cl.set_time_ms(t_info.unwrap().length as i64);
            queue_string.push_str(&format!(
                "{}. [{}]({}) | `{}` \n\n",
                num,
                t_info.unwrap().title,
                t_info.unwrap().uri,
                cl.get_time()
            ));
        }

        eb.field("Next Songs", queue_string, false);
    }

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.0 = eb.0;
            e
        })
    })
    .await?;

    Ok(())
}

async fn queue_checker(ctx: Context, guild_id: GuildId, data: Arc<crate::Data>) {
    loop {
        sleep(Duration::from_secs(60)).await;
        {
            if data.voice_timer_map.get(&guild_id).is_some() {
                return;
            }

            let nodes = data.lavalink.nodes().await;
            let node = match nodes.get(&guild_id.0) {
                Some(node) => node,
                None => return,
            };

            if node.queue.is_empty() && node.now_playing.is_none() {
                let ctx_clone = ctx.clone();
                let data = Arc::clone(&data);
                tokio::spawn(async move {
                    voice_utils::create_new_timer(ctx_clone, guild_id, data).await;
                });
                return;
            }
        }
    }
}

/// Either clears the entire queue, or removes a specific track
#[poise::command(slash_command, aliases("c"))]
pub async fn clear(ctx: crate::Context<'_>) -> CommandResult {
    let guild = ctx.guild().await.unwrap();

    if !get_voice_state(ctx, &guild).await? {
        poise::say_reply(
            ctx,
            "Please be in a voice channel or in the same voice channel as me!".into(),
        )
        .await?;
        return Ok(());
    }

    let lava_client = &ctx.data().lavalink;

    let nodes = lava_client.nodes().await;
    let mut node = match nodes.get_mut(&ctx.guild_id().unwrap().0) {
        Some(node) => node,
        None => {
            poise::say_reply(
                ctx,
                "The bot isn't connected to a voice channel or node! Please re-run join or play!"
                    .into(),
            )
            .await?;
            return Ok(());
        }
    };

    if !permissions_helper::check_permission_2(ctx, None, false).await? {
        poise::say_reply(
            ctx,
            JesterError::PermissionError(PermissionType::UserPerm("manage messages")).to_string(),
        )
        .await?;
    } else {
        node.queue.drain(1..);

        react(ctx, "💣").await?;
    }

    Ok(())
}

/// Remove a track from the queue
#[poise::command(slash_command, aliases("r"))]
pub async fn remove(
    ctx: crate::Context<'_>,
    #[description = "Track number"] clear_num: usize,
) -> CommandResult {
    let guild = ctx.guild().await.unwrap();

    if !get_voice_state(ctx, &guild).await? {
        poise::say_reply(
            ctx,
            "Please be in a voice channel or in the same voice channel as me!".into(),
        )
        .await?;
        return Ok(());
    }

    let lava_client = &ctx.data().lavalink;

    let nodes = lava_client.nodes().await;
    let mut node = match nodes.get_mut(&ctx.guild_id().unwrap().0) {
        Some(node) => node,
        None => {
            poise::say_reply(
                ctx,
                "The bot isn't connected to a voice channel or node! Please re-run join or play!"
                    .into(),
            )
            .await?;
            return Ok(());
        }
    };

    if clear_num == 0 {
        poise::say_reply(
            ctx,
            JesterError::MissingError("number greater than 0").to_string(),
        )
        .await?;

        return Ok(());
    }

    let queue = &mut node.queue;

    let track_queue = match queue.get(clear_num) {
        Some(track_queue) => track_queue,
        None => {
            poise::say_reply(ctx, "This number doesn't exist in the queue!".into()).await?;

            return Ok(());
        }
    };

    let name = track_queue.track.info.as_ref().unwrap().title.clone();

    queue.remove(clear_num);

    poise::say_reply(ctx, format!("Successfully removed track {}", name)).await?;

    Ok(())
}

/// Skips the current track. If there are no tracks in the queue, the player is stopped
#[poise::command(slash_command, aliases("s"))]
pub async fn skip(ctx: crate::Context<'_>) -> CommandResult {
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

    let lava_client = &ctx.data().lavalink;

    if !lava_client.nodes().await.contains_key(&guild_id.0) {
        poise::say_reply(
                ctx,
                "The bot isn't connected to a voice channel or playing anything! Please re-run join or play!".into(),
            )
            .await?;
        return Ok(());
    }

    if lava_client.skip(guild_id).await.is_some() {
        let nodes = lava_client.nodes().await;
        let node = nodes.get(&ctx.guild_id().unwrap().0).unwrap();

        if node.queue.is_empty() && node.now_playing.is_none() {
            lava_client.stop(guild_id).await?;
        }
    }

    react(ctx, "⏭️").await?;

    Ok(())
}

/// Seeks in the current track using hh:mm:ss format. mm:ss is also supported
#[poise::command(slash_command)]
pub async fn seek(ctx: crate::Context<'_>, #[description = "Time"] time: String) -> CommandResult {
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

    let time = match command_utils::deconstruct_time(time) {
        Ok(time) => time,
        Err(e) => {
            poise::say_reply(
                ctx,
                JesterError::MissingError(&format!("valid amount of {}", e)).to_string(),
            )
            .await?;

            return Ok(());
        }
    };

    let lava_client = &ctx.data().lavalink;

    if !lava_client.nodes().await.contains_key(&guild_id.0) {
        poise::say_reply(
                ctx,
                "The bot isn't connected to a voice channel or playing anything! Please re-run join or play!".into(),
            )
            .await?;
        return Ok(());
    };

    lava_client
        .seek(guild_id, Duration::from_secs(time))
        .await?;

    poise::say_reply(ctx, "Seeking!".into()).await?;

    Ok(())
}

pub async fn music_help(ctx: crate::Context<'_>) {
    let content = concat!(
        "play <URL or search keywords> : Plays the specified track \n\n",
        "pause: Pauses the current track \n\n",
        "resume <author> <text>: Resumes the current track \nAlias: unpause \n\n",
        "stop: Stops the current track and empties the queue. Doesn't disconnect the bot \n\n",
        "skip: Skips the current track. If there are no tracks in the queue, the player is stopped \n\n",
        "seek <time>: Seeks in the current track using hh:mm:ss format. mm:ss is also supported",
        "clear (track number): either clears the entire queue, or removes a specific track",
        "queue: See the current queue for the guild and what's playing");

    let _ = poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("Music Help");
            e.description("Description: Commands for playing music");
            e.field("Commands", content, false);
            e.footer(|f| {
                f.text("For more information on voice commands, please check voice help");
                f
            });
            e
        })
    })
    .await;
}
