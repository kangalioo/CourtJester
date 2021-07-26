use crate::helpers::reaction_gifs::{add_to_cache, check_image_cache, fetch_gifs};
use crate::{Context, PrefixContext};
use rand::{prelude::StdRng, Rng, SeedableRng};
use serenity::{framework::standard::CommandResult, model::prelude::*};

enum Recipient {
    SomeoneElse(String),
    Yourself,
    Everyone,
}

#[allow(clippy::manual_map)] // it's more readable here
fn extract_recipient(ctx: PrefixContext<'_>, recipient: &str) -> Option<Recipient> {
    if recipient.eq_ignore_ascii_case("everyone") {
        Some(Recipient::Everyone)
    } else if let Some(mention) = ctx.msg.mentions.get(0) {
        if mention.id == ctx.msg.author.id {
            Some(Recipient::Yourself)
        } else {
            Some(Recipient::SomeoneElse(mention.name.clone()))
        }
    } else {
        None
    }
}

#[poise::command(track_edits, rename = "hug")]
pub async fn prefix_hug(ctx: PrefixContext<'_>, #[rest] recipient: String) -> CommandResult {
    match extract_recipient(ctx, &recipient) {
        Some(recipient) => {
            hug_inner(Context::Prefix(ctx), recipient).await?;
        }
        None => {
            let help_msg =
                "You want to give a hug? Please mention who you want to hug or provide `everyone`!";
            poise::say_prefix_reply(ctx, help_msg.into()).await?;
        }
    }
    Ok(())
}

/// Gives a hug to someone
#[poise::command(slash_command, rename = "hug")]
pub async fn slash_hug(
    ctx: Context<'_>,
    #[description = "Who should receive the hug?"] recipient: Member,
) -> CommandResult {
    let recipient = if recipient.user.id == ctx.author().id {
        Recipient::Yourself
    } else {
        Recipient::SomeoneElse(recipient.user.name)
    };
    hug_inner(ctx, recipient).await
}

async fn hug_inner(ctx: Context<'_>, recipient: Recipient) -> CommandResult {
    let message = match recipient {
        Recipient::Everyone => "Group hug!".to_owned(),
        Recipient::Yourself => "You hugged yourself. Cute 🙂".to_owned(),
        Recipient::SomeoneElse(name) => format!("{} hugs {}", ctx.author().name, name),
    };

    let raw_gifs = fetch_gifs(ctx, "anime hug", 10, "medium").await?;
    let mut rng = StdRng::from_entropy();

    let guild_id = ctx.guild_id().unwrap();
    let gifs = check_image_cache(ctx, guild_id, "hug".to_owned(), raw_gifs).await;

    let val = rng.gen_range(0..=gifs.len() - 1);

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.color(0xed9e2f);
            e.description(message);
            e.image(&gifs[val].media[0].get("gif").unwrap().url);
            e
        })
    })
    .await?;

    add_to_cache(ctx, guild_id, "hug".to_owned(), gifs[val].url.to_owned()).await;

    Ok(())
}

#[poise::command(track_edits, rename = "pat")]
pub async fn prefix_pat(ctx: PrefixContext<'_>, #[rest] recipient: String) -> CommandResult {
    match extract_recipient(ctx, &recipient) {
        Some(recipient) => {
            pat_inner(Context::Prefix(ctx), recipient).await?;
        }
        None => {
            let help_msg = "I wanna pat someone! Please mention who to pat or provide `everyone`!";
            poise::say_prefix_reply(ctx, help_msg.into()).await?;
        }
    }
    Ok(())
}

/// Gives a pat to someone
#[poise::command(slash_command, rename = "pat")]
pub async fn slash_pat(
    ctx: Context<'_>,
    #[description = "Who should receive the pat?"] recipient: Member,
) -> CommandResult {
    let recipient = if recipient.user.id == ctx.author().id {
        Recipient::Yourself
    } else {
        Recipient::SomeoneElse(recipient.user.name)
    };
    pat_inner(ctx, recipient).await
}

async fn pat_inner(ctx: Context<'_>, recipient: Recipient) -> CommandResult {
    let raw_gifs = fetch_gifs(ctx, "anime pat", 10, "medium").await?;
    let mut rng = StdRng::from_entropy();

    let guild_id = ctx.guild_id().unwrap();
    let gifs = check_image_cache(ctx, guild_id, "pat".to_owned(), raw_gifs).await;

    let val = rng.gen_range(0..=gifs.len() - 1);

    let message = match recipient {
        Recipient::Everyone => "Pats for everyone!".to_owned(),
        Recipient::Yourself => "You gave yourself a pat on the back!".to_owned(),
        Recipient::SomeoneElse(name) => format!("{} pats {}", ctx.author().name, name),
    };

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.color(0x27e6d9);
            e.description(message);
            e.image(&gifs[val].media[0].get("gif").unwrap().url);
            e
        })
    })
    .await?;

    add_to_cache(ctx, guild_id, "pat".to_owned(), gifs[val].url.to_owned()).await;

    Ok(())
}

#[poise::command(track_edits, rename = "slap")]
pub async fn prefix_slap(ctx: PrefixContext<'_>, #[rest] recipient: String) -> CommandResult {
    match extract_recipient(ctx, &recipient) {
        Some(recipient) => {
            slap_inner(Context::Prefix(ctx), recipient).await?;
        }
        None => {
            let help_msg =
                "Wait... who do I slap again? Please mention the person or provide `everyone`!";
            poise::say_prefix_reply(ctx, help_msg.into()).await?;
        }
    }
    Ok(())
}

/// Slaps someone
#[poise::command(slash_command, rename = "slap")]
pub async fn slash_slap(
    ctx: Context<'_>,
    #[description = "Who should receive the slap?"] recipient: Member,
) -> CommandResult {
    let recipient = if recipient.user.id == ctx.author().id {
        Recipient::Yourself
    } else {
        Recipient::SomeoneElse(recipient.user.name)
    };
    slap_inner(ctx, recipient).await
}

async fn slap_inner(ctx: Context<'_>, recipient: Recipient) -> CommandResult {
    let raw_gifs = fetch_gifs(ctx, "anime slap", 10, "medium").await?;
    let mut rng = StdRng::from_entropy();

    let guild_id = ctx.guild_id().unwrap();
    let gifs = check_image_cache(ctx, guild_id, "slap".to_owned(), raw_gifs).await;

    let val = rng.gen_range(0..=gifs.len() - 1);

    let message = match recipient {
        Recipient::Everyone => "You slapped everyone! Ouch... that's gotta hurt.".to_owned(),
        Recipient::Yourself => {
            "You slapped yourself? Not sure if that's a good or bad thing...".to_owned()
        }
        Recipient::SomeoneElse(name) => format!("{} slaps {}", ctx.author().name, name),
    };

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.color(0xd62929);
            e.description(message);
            e.image(&gifs[val].media[0].get("gif").unwrap().url);
            e
        })
    })
    .await?;

    add_to_cache(ctx, guild_id, "slap".to_owned(), gifs[val].url.to_owned()).await;

    Ok(())
}

#[poise::command(track_edits, rename = "kiss")]
pub async fn prefix_kiss(ctx: PrefixContext<'_>, #[rest] recipient: String) -> CommandResult {
    match extract_recipient(ctx, &recipient) {
        Some(recipient) => {
            kiss_inner(Context::Prefix(ctx), recipient).await?;
        }
        None => {
            let help_msg = "You want to express your feelings? Please mention who you want to kiss or provide `everyone`!";
            poise::say_prefix_reply(ctx, help_msg.into()).await?;
        }
    }
    Ok(())
}

/// Kisses someone
#[poise::command(slash_command, rename = "kiss")]
pub async fn slash_kiss(
    ctx: Context<'_>,
    #[description = "Who should receive the kiss?"] recipient: Member,
) -> CommandResult {
    let recipient = if recipient.user.id == ctx.author().id {
        Recipient::Yourself
    } else {
        Recipient::SomeoneElse(recipient.user.name)
    };
    kiss_inner(ctx, recipient).await
}

async fn kiss_inner(ctx: Context<'_>, recipient: Recipient) -> CommandResult {
    let raw_gifs = fetch_gifs(ctx, "anime kiss", 10, "medium").await?;
    let mut rng = StdRng::from_entropy();

    let guild_id = ctx.guild_id().unwrap();
    let gifs = check_image_cache(ctx, guild_id, "kiss".to_owned(), raw_gifs).await;

    let val = rng.gen_range(0..=gifs.len() - 1);

    let message = match recipient {
        Recipient::Everyone => "A friendly kiss to everyone!".to_owned(),
        Recipient::Yourself => "Well... You just kissed yourself".to_owned(),
        Recipient::SomeoneElse(name) => format!("{} kisses {}", ctx.author().name, name),
    };

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.color(0xffb6c1);
            e.description(message);
            e.image(&gifs[val].media[0].get("gif").unwrap().url);
            e
        })
    })
    .await?;

    add_to_cache(ctx, guild_id, "kiss".to_owned(), gifs[val].url.to_owned()).await;

    Ok(())
}

/// Empthasizes that you're disgusted
#[poise::command(slash_command, track_edits)]
pub async fn disgust(ctx: Context<'_>) -> CommandResult {
    let raw_gifs = fetch_gifs(ctx, "anime disgust", 10, "medium").await?;
    let mut rng = StdRng::from_entropy();

    let guild_id = ctx.guild_id().unwrap();
    let gifs = check_image_cache(ctx, guild_id, "disgust".to_owned(), raw_gifs).await;

    let val = rng.gen_range(0..=gifs.len() - 1);

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.color(0x50c878);
            e.description(format!("{} is disgusted 😕", ctx.author().name));
            e.image(&gifs[val].media[0].get("gif").unwrap().url);
            e
        })
    })
    .await?;

    add_to_cache(
        ctx,
        guild_id,
        "disgust".to_owned(),
        gifs[val].url.to_owned(),
    )
    .await;

    Ok(())
}

/// Emphasizes that you're crying
#[poise::command(slash_command, track_edits)]
pub async fn cry(ctx: Context<'_>) -> CommandResult {
    let raw_gifs = fetch_gifs(ctx, "anime cry", 10, "medium").await?;
    let mut rng = StdRng::from_entropy();

    let guild_id = ctx.guild_id().unwrap();
    let gifs = check_image_cache(ctx, guild_id, "cry".to_owned(), raw_gifs).await;

    let val = rng.gen_range(0..=gifs.len() - 1);

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.color(0x3252e3);
            e.description(format!("{} is crying! 😭", ctx.author().name));
            e.image(&gifs[val].media[0].get("gif").unwrap().url);
            e
        })
    })
    .await?;

    add_to_cache(ctx, guild_id, "cry".to_owned(), gifs[val].url.to_owned()).await;

    Ok(())
}

/// Emphasizes that something is cringey
#[poise::command(slash_command, track_edits)]
pub async fn cringe(ctx: Context<'_>) -> CommandResult {
    let raw_gifs = fetch_gifs(ctx, "cringe", 10, "low").await?;
    let mut rng = StdRng::from_entropy();

    let guild_id = ctx.guild_id().unwrap();
    let gifs = check_image_cache(ctx, guild_id, "cringe".to_owned(), raw_gifs).await;

    let val = rng.gen_range(0..=gifs.len() - 1);

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.color(0x634644);
            e.description(format!(
                "{} thinks that's really cringey 😬",
                ctx.author().name
            ));
            e.image(&gifs[val].media[0].get("gif").unwrap().url);
            e
        })
    })
    .await?;

    add_to_cache(ctx, guild_id, "cringe".to_owned(), gifs[val].url.to_owned()).await;

    Ok(())
}

/// Searches for a gif with a search string
#[poise::command(slash_command, track_edits, aliases("gif"))]
pub async fn gifsearch(
    ctx: Context<'_>,
    #[description = "What kind of gif are you looking for?"]
    #[rest]
    search_string: String,
) -> CommandResult {
    let filter = if ctx
        .channel_id()
        .to_channel(ctx.discord())
        .await
        .unwrap()
        .is_nsfw()
    {
        "off"
    } else {
        "medium"
    };

    let gifs = fetch_gifs(ctx, &search_string, 10, filter).await?;
    let mut rng = StdRng::from_entropy();
    let val = rng.gen_range(0..=gifs.len() - 1);

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.color(0x5ed13b);
            e.image(&gifs[val].media[0].get("gif").unwrap().url);
            e
        })
    })
    .await?;

    Ok(())
}

pub async fn image_help(ctx: crate::Context<'_>) {
    let content = concat!(
        "gif: Fetches a random gif from tenor \nNote: The content filter is turned off in an NSFW channel \n\n",
        "hug <mention>: Gives wholesome hugs to someone \n\n",
        "pat <mention>: Pats someone on the head \n\n",
        "slap <mention>: Give someone a slap \n\n",
        "kiss <mention>: You already know what this is and I am shaking my head... \n\n",
        "cry: Emphasizes that you're crying  \n\n",
        "cringe: Emphasizes that something is cringey \n\n");

    let _ = poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("Images/Reaction Help");
            e.description("Description: Various commands that work with images");
            e.field("Commands", content, false);
            e
        })
    })
    .await;
}
