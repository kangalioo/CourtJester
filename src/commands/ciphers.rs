use serenity::{framework::standard::CommandResult, model::prelude::*};

use crate::{Context, JesterError};

/// Encodes a message in base64
/// Usage `b64encode <message>`
#[poise::command(slash_command, track_edits)]
pub async fn b64encode(
    ctx: Context<'_>,
    #[rest]
    #[description = "Plain string to encode"]
    string: String,
) -> CommandResult {
    let b64_string = base64::encode(string);

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("Base64 Engine");
            e.description(format!("Encoded Message: `{}`", b64_string));
            e
        })
    })
    .await?;
    Ok(())
}

/// Decodes a message in base64
/// Usage `b64encode <message>`
#[poise::command(slash_command, track_edits)]
pub async fn b64decode(
    ctx: Context<'_>,
    #[rest]
    #[description = "Base64-encoded payload to decode"]
    string: String,
) -> CommandResult {
    let b64_bytes = match base64::decode(string) {
        Ok(bytes) => bytes,
        Err(_error) => {
            poise::say_reply(ctx, JesterError::MissingError("base64 string").to_string()).await?;
            return Ok(());
        }
    };

    let decoded_string = String::from_utf8(b64_bytes).unwrap();

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("Base64 Engine");
            e.description(format!("Decoded Message: `{}`", decoded_string));
            e
        })
    })
    .await?;

    Ok(())
}

pub async fn cipher_help(ctx: &serenity::prelude::Context, channel_id: ChannelId) {
    let content = concat!(
        "b64encode <message>: Encodes a message in base64 \n\n",
        "b64decode <b64 string>: Decodes a base64 message"
    );

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Cipher Help");
                e.description("Description: Encoding/Decoding messages");
                e.field("Commands", content, false);
                e
            })
        })
        .await;
}
