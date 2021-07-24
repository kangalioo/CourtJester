use std::borrow::Cow;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    http::AttachmentType,
    model::prelude::*,
    prelude::*,
};

use crate::JesterError;

/// Gets your own, or the mentioned person's avatar
#[poise::command(slash_command, track_edits)]
pub async fn avatar(
    ctx: crate::Context<'_>,
    #[description = "Whose user's avatar do you want to see?"] user: Option<Member>,
) -> CommandResult {
    let user = match &user {
        Some(user) => &user.user,
        None => ctx.author(),
    };

    poise::say_reply(ctx, user.face()).await?;

    Ok(())
}

#[command]
#[aliases("steal")]
#[required_permissions("MANAGE_EMOJIS_AND_STICKERS")]
pub async fn kang(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let emoji = match args.single::<EmojiIdentifier>() {
        Ok(id) => id,
        Err(_) => {
            msg.channel_id
                .say(ctx, JesterError::MissingError("custom emoji"))
                .await?;

            return Ok(());
        }
    };

    let guild = msg.guild(ctx).await.unwrap();
    if guild.emojis.contains_key(&emoji.id) {
        msg.channel_id
            .say(ctx, "This emoji already exists in this server! Aborting...")
            .await?;

        return Ok(());
    }

    let emoji_url = emoji.url();

    let url_length = emoji_url.len();
    let ext = &emoji_url[url_length - 3..url_length];

    let image_bytes = reqwest::get(&emoji.url()).await?.bytes().await?;
    let encoded_bytes = base64::encode(image_bytes);
    let formatted_bytes = format!("data:image/{};base64,{}", ext, encoded_bytes);

    let name = args.single::<String>().unwrap_or(emoji.name);

    match guild.create_emoji(ctx, &name, &formatted_bytes).await {
        Ok(new_emoji) => {
            msg.channel_id
                .say(
                    ctx,
                    format!("New emoji {} created! {}", new_emoji.name, new_emoji),
                )
                .await?;

            Ok(())
        }
        Err(e) => {
            msg.channel_id.say(
                ctx,
                "Something went wrong with emoji creation. Check your emoji limit? The error message is below."
            ).await?;

            Err(e.into())
        }
    }
}

/// Get the information of an emoji
#[poise::command(slash_command, track_edits, aliases("einfo"))]
pub async fn emoji_info(
    ctx: crate::Context<'_>,
    #[description = "Which emoji to show information about"] emoji: EmojiIdentifier,
) -> CommandResult {
    let emoji_url = emoji.url();

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("Emoji info for...");
            e.thumbnail(&emoji_url);
            e.field("Name", emoji.name, false);
            e.field("Emoji ID", emoji.id.0, false);
            e.field("Image URL", format!("[Click here]({})", &emoji_url), false);
            e.footer(|f| {
                f.text(format!(
                    "Requested by {}#{}",
                    ctx.author().name,
                    ctx.author().discriminator
                ));
                f
            });
            e
        })
    })
    .await?;

    Ok(())
    // Embed with emoji name, image as thumbnail, and original link to image
}

#[poise::command]
pub async fn spoiler(ctx: crate::PrefixContext<'_>) -> CommandResult {
    let attachment = match ctx.msg.attachments.get(0) {
        Some(attachment) => attachment,
        None => {
            poise::say_prefix_reply(ctx, JesterError::MissingError("attachment").to_string())
                .await?;

            return Ok(());
        }
    };

    let new_filename = format!("SPOILER_{}", attachment.filename);

    let bytes = attachment.download().await?;

    let new_attachment = AttachmentType::Bytes {
        data: Cow::from(bytes),
        filename: new_filename.to_owned(),
    };

    let msg_result = poise::send_prefix_reply(ctx, |m| {
        m.content(format!("Invoked by {}", ctx.msg.author.mention()));
        m.attachment(new_attachment);
        m
    })
    .await;

    if msg_result.is_err() {
        poise::say_prefix_reply(
            ctx,
            "This file is too big! Please attach a file less than 8 MB...".into(),
        )
        .await?;

        return Ok(());
    }

    if ctx.msg.delete(ctx.discord).await.is_err() {
        poise::say_prefix_reply(
            ctx,
            concat!("The spoiled attachment was posted, but I cannot delete the old message! \n",
            "Please give me the `MANAGE_MESSAGES` permission if you want the unspoiled image deleted!").into()
        ).await?;

        return Ok(());
    };

    Ok(())
}

pub async fn utility_help(ctx: &Context, channel_id: ChannelId) {
    let content = concat!(
        "avatar (user mention/ID): Gets your own, or the mentioned person's avatar \n\n",
        "spoiler <attachment>: Creates a spoiler from an attached file \n\n",
        "kang <emoji> (new name): Steal an emoji from anywhere and load it to your server. Requires the `manage emojis` permission \n\n",
        "einfo <emoji>: Get the information of an emoji"
    );

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Miscellaneous Utility Help");
                e.description("Description: Various utility commands");
                e.field("Commands", content, false);
                e
            })
        })
        .await;
}
