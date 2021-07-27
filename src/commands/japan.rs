use reqwest::Url;
use serde::{Deserialize, Serialize};
use serenity::{framework::standard::CommandResult, model::prelude::*, prelude::*};
use std::fmt::Write;
use std::time::Duration;

use crate::{
    helpers::embed_store,
    structures::{errors::JesterError, AnimeResult, MangaResult},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    results: Vec<ResultType>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResultType {
    Anime(AnimeResult),
    Manga(MangaResult),
}

impl ResultType {
    fn unwrap_anime(&self) -> Option<&AnimeResult> {
        if let ResultType::Anime(a) = self {
            Some(a)
        } else {
            None
        }
    }

    fn unwrap_manga(&self) -> Option<&MangaResult> {
        if let ResultType::Manga(m) = self {
            Some(m)
        } else {
            None
        }
    }
}

#[poise::command]
pub async fn anime(ctx: crate::PrefixContext<'_>, #[rest] title: String) -> CommandResult {
    let msg = ctx.msg;

    let results = match fetch_info(ctx, "anime", &title).await {
        Ok(info) => info.results,
        Err(_) => {
            msg.channel_id
                .say(ctx.discord, "Couldn't find your request on MAL!")
                .await?;

            return Ok(());
        }
    };

    let animes = results
        .iter()
        .filter_map(ResultType::unwrap_anime)
        .collect::<Vec<_>>();

    let result_string = animes
        .iter()
        .enumerate()
        .fold(String::new(), |mut acc, (num, anime)| {
            let _ = writeln!(&mut acc, "{}. {}\n", num + 1, anime.title);
            acc
        });

    let result_embed = embed_store::get_result_embed(&result_string);

    let sent_message = msg
        .channel_id
        .send_message(ctx.discord, |m| {
            m.embed(|e| {
                e.0 = result_embed.0;
                e
            })
        })
        .await?;

    while let Ok(value) = ask_for_results(ctx.discord, msg).await {
        let index = value as usize;

        if let Some(anime) = animes.get(index - 1) {
            let anime_embed = embed_store::get_anime_embed(anime);

            msg.channel_id
                .send_message(ctx.discord, |m| {
                    m.embed(|e| {
                        e.0 = anime_embed.0;
                        e
                    })
                })
                .await?;

            break;
        }
    }

    sent_message.delete(ctx.discord).await?;

    Ok(())
}

#[poise::command]
pub async fn manga(ctx: crate::PrefixContext<'_>, #[rest] title: String) -> CommandResult {
    let msg = ctx.msg;

    let results = match fetch_info(ctx, "manga", &title).await {
        Ok(info) => info.results,
        Err(_) => {
            msg.channel_id
                .say(ctx.discord, "Couldn't find your request on MAL!")
                .await?;

            return Ok(());
        }
    };

    let mangas = results
        .iter()
        .filter_map(ResultType::unwrap_manga)
        .collect::<Vec<_>>();

    let result_string = mangas
        .iter()
        .enumerate()
        .fold(String::new(), |mut acc, (num, manga)| {
            let _ = writeln!(&mut acc, "{}. {}\n", num + 1, manga.title);
            acc
        });

    let result_embed = embed_store::get_result_embed(&result_string);

    let sent_message = msg
        .channel_id
        .send_message(ctx.discord, |m| {
            m.embed(|e| {
                e.0 = result_embed.0;
                e
            })
        })
        .await?;

    while let Ok(value) = ask_for_results(ctx.discord, msg).await {
        let index = value as usize;

        if let Some(manga) = mangas.get(index - 1) {
            let manga_embed = embed_store::get_manga_embed(manga);

            msg.channel_id
                .send_message(ctx.discord, |m| {
                    m.embed(|e| {
                        e.0 = manga_embed.0;
                        e
                    })
                })
                .await?;

            break;
        }
    }

    sent_message.delete(ctx.discord).await?;

    Ok(())
}

async fn ask_for_results(ctx: &Context, msg: &Message) -> CommandResult<isize> {
    let channel_id = msg.channel_id;

    let result = msg
        .author
        .await_reply(ctx)
        .filter(move |given_msg| given_msg.channel_id == channel_id)
        .timeout(Duration::from_secs(30))
        .await;

    match result {
        Some(recieved_msg) => {
            if recieved_msg.content == "abort" {
                let _ = recieved_msg
                    .channel_id
                    .say(ctx, "Aborting...")
                    .await;

                return Err("Aborted".into());
            }

            match recieved_msg.content.parse::<isize>() {
                Ok(num) => Ok(num),
                Err(_) => Ok(-1),
            }
        }
        None => {
            let _ = channel_id.say(ctx, "Timed out").await;

            Err("Timeout".into())
        }
    }
}

async fn fetch_info(
    ctx: crate::PrefixContext<'_>,
    search_type: &str,
    search: &str,
) -> CommandResult<Response> {
    let reqwest_client = &ctx.data.reqwest_client;

    let url = Url::parse_with_params(
        &format!("https://api.jikan.moe/v3/search/{}", search_type),
        &[("q", search), ("limit", "5")],
    )?;

    let resp = reqwest_client
        .get(url)
        .send()
        .await?
        .json::<Response>()
        .await?;

    Ok(resp)
}

pub async fn japan_help(ctx: crate::Context<'_>) {
    let content = concat!(
        "anime <title>: Searches for an anime's information from the title \n\n",
        "manga <title>: Searches for a manga's information from the title"
    );

    let _ = poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("Japan Help");
            e.description("Description: Commands that deal with japanese media");
            e.field("Commands", content, false);
            e
        })
    })
    .await;
}
