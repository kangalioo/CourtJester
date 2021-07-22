use reqwest::Url;
use serde::Deserialize;
use serenity::{framework::standard::CommandResult, model::id::GuildId};

use crate::{structures::GifResult, Context};

#[derive(Debug, Deserialize)]
struct Response {
    results: Vec<GifResult>,
}

pub async fn fetch_gifs(
    ctx: Context<'_>,
    search: &str,
    amount: usize,
    filter: &str,
) -> CommandResult<Vec<GifResult>> {
    let tenor_key = ctx.data().pub_creds.get("tenor").unwrap();

    let url = Url::parse_with_params(
        "https://api.tenor.com/v1/search",
        &[
            ("q", search),
            ("key", tenor_key),
            ("limit", &format!("{}", amount)),
            ("contentfilter", filter),
        ],
    )?;

    let resp = ctx
        .data()
        .reqwest_client
        .get(url)
        .send()
        .await?
        .json::<Response>()
        .await?;

    Ok(resp.results)
}

pub async fn add_to_cache(ctx: Context<'_>, guild_id: GuildId, key: String, url: String) {
    let image_cache = &ctx.data().reaction_image_cache;

    let cached_url = image_cache.get(&(guild_id, key.to_owned()));

    match cached_url {
        Some(cached_url) => {
            if cached_url.value() != &url {
                drop(cached_url);

                image_cache.insert((guild_id, key), url);
            }
        }
        None => {
            image_cache.insert((guild_id, key), url);
        }
    }
}

pub async fn check_image_cache(
    ctx: Context<'_>,
    guild_id: GuildId,
    search_key: String,
    mut gifs: Vec<GifResult>,
) -> Vec<GifResult> {
    let cached_url = match ctx.data().reaction_image_cache.get(&(guild_id, search_key)) {
        Some(image_struct) => image_struct,
        None => return gifs,
    };

    if let Some(index) = gifs.iter().position(|gif| &gif.url == cached_url.value()) {
        gifs.remove(index);
    };

    gifs
}
