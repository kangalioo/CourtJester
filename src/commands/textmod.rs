use serenity::{framework::standard::CommandResult, model::prelude::*};

use crate::helpers::*;

async fn get_last_message(ctx: crate::Context<'_>) -> CommandResult<Message> {
    let message_iterator = match ctx {
        crate::Context::Prefix(ctx) => {
            ctx.msg
                .channel_id
                .messages(ctx.discord, |retriever| {
                    retriever.before(ctx.msg.id).limit(1)
                })
                .await?
        }
        crate::Context::Slash(ctx) => {
            ctx.interaction
                .channel_id
                .messages(ctx.discord, |f| f.limit(1))
                .await?
        }
    };
    Ok(message_iterator.into_iter().next().unwrap())
}

/// Outputs a spongebob mock string
///
/// Usage: `mock <message>`
#[poise::command(slash_command, track_edits)]
pub async fn mock(
    ctx: crate::Context<'_>,
    #[description = "Your text"]
    #[rest]
    string: String,
) -> CommandResult {
    let mock_string = textmod_helper::get_mock_string(&string);

    poise::say_reply(ctx, mock_string).await?;

    Ok(())
}

/// Like mock, but uses the last message as text
#[poise::command(slash_command, track_edits)]
pub async fn mockl(ctx: crate::Context<'_>) -> CommandResult {
    let input_message = get_last_message(ctx).await?;

    let mock_string = textmod_helper::get_mock_string(&input_message.content);

    poise::say_reply(ctx, mock_string).await?;

    Ok(())
}

/// Inverts the characters in a string
///
/// Usage: `inv <message>`
#[poise::command(slash_command, track_edits)]
pub async fn inv(
    ctx: crate::Context<'_>,
    #[description = "Your text"]
    #[rest]
    string: String,
) -> CommandResult {
    let inv_string = textmod_helper::get_inverted_string(&string);

    poise::say_reply(ctx, inv_string).await?;

    Ok(())
}

/// Like inv, but uses the last message as text
#[poise::command(slash_command, track_edits)]
pub async fn invl(ctx: crate::Context<'_>) -> CommandResult {
    let input_message = get_last_message(ctx).await?;

    let inv_string = textmod_helper::get_inverted_string(&input_message.content);

    poise::say_reply(ctx, inv_string).await?;

    Ok(())
}

/// Converts the provided string to uppercase letters
///
/// Usage: `upp <message>`
#[poise::command(slash_command, track_edits)]
pub async fn upp(
    ctx: crate::Context<'_>,
    #[description = "Your text"]
    #[rest]
    string: String,
) -> CommandResult {
    poise::say_reply(ctx, string.to_uppercase()).await?;

    Ok(())
}

/// Like upp, but uses the last message as text
#[poise::command(slash_command, track_edits)]
pub async fn uppl(ctx: crate::Context<'_>) -> CommandResult {
    let input_message = get_last_message(ctx).await?;

    poise::say_reply(ctx, input_message.content.to_uppercase()).await?;

    Ok(())
}

/// Converts the provided string to lowercase
///
/// Usage: `low <message>`
#[poise::command(slash_command, track_edits)]
pub async fn low(
    ctx: crate::Context<'_>,
    #[description = "Your text"]
    #[rest]
    string: String,
) -> CommandResult {
    poise::say_reply(ctx, string.to_lowercase()).await?;

    Ok(())
}

/// Like low, but uses the last message as text
#[poise::command(slash_command, track_edits)]
pub async fn lowl(ctx: crate::Context<'_>) -> CommandResult {
    let input_message = get_last_message(ctx).await?;

    poise::say_reply(ctx, input_message.content.to_lowercase()).await?;

    Ok(())
}

/// Puts a random amount of spaces between each character of the message
///
/// Usage: `space <message>`
#[poise::command(slash_command, track_edits)]
pub async fn space(
    ctx: crate::Context<'_>,
    #[description = "Your text"]
    #[rest]
    string: String,
) -> CommandResult {
    let spaced_string = textmod_helper::get_spaced_string(&string, false);

    poise::say_reply(ctx, spaced_string).await?;

    Ok(())
}

/// Like space, but uses the last message as text
#[poise::command(slash_command, track_edits)]
pub async fn spacel(ctx: crate::Context<'_>) -> CommandResult {
    let input_message = get_last_message(ctx).await?;

    let spaced_string = textmod_helper::get_spaced_string(&input_message.content, false);

    poise::say_reply(ctx, spaced_string).await?;

    Ok(())
}

/// Similar to space, but puts a larger amount of space between each character
///
/// Usage: `biggspace <message>`
#[poise::command(slash_command, track_edits, aliases("bigspace"))]
pub async fn biggspace(
    ctx: crate::Context<'_>,
    #[description = "Your text"]
    #[rest]
    string: String,
) -> CommandResult {
    let bigspace_string = textmod_helper::get_spaced_string(&string, true);

    poise::say_reply(ctx, bigspace_string).await?;

    Ok(())
}

/// Like biggspace, but uses the last message as text
#[poise::command(slash_command, track_edits)]
pub async fn biggspacel(ctx: crate::Context<'_>) -> CommandResult {
    let input_message = get_last_message(ctx).await?;

    let bigspace_string = textmod_helper::get_spaced_string(&input_message.content, true);

    poise::say_reply(ctx, bigspace_string).await?;

    Ok(())
}

/// Become a hackerman by making h4ck3d w0rd5
#[poise::command(slash_command, track_edits)]
pub async fn h4ck(
    ctx: crate::Context<'_>,
    #[description = "Your text"]
    #[rest]
    string: String,
) -> CommandResult {
    let hacked_string = textmod_helper::get_hacked_string(&string);

    poise::say_reply(ctx, hacked_string).await?;

    Ok(())
}

/// Translate to the uwu wanguwage uwu
#[poise::command(slash_command, track_edits)]
pub async fn uwu(
    ctx: crate::Context<'_>,
    #[description = "Your text"]
    #[rest]
    string: String,
) -> CommandResult {
    let uwu_string = textmod_helper::get_uwu_string(&string);

    poise::say_reply(ctx, uwu_string).await?;

    Ok(())
}

pub async fn textmod_help(ctx: crate::Context<'_>) {
    let content = concat!(
        "mock <message>: Spongebob mocks a string \n\n",
        "inv <message>: Inverts capitalization of each letter in the message \n\n",
        "upp <message>: Every letter becomes uppercase \n\n",
        "low <message>: Every letter becomes lowercase \n\n",
        "space <message>: Spaces out each letter in the message (whitespace omitted) \n\n",
        "biggspace <message>: Same as space, but W I D E R \n\n",
        "h4ck <message>: Become a hackerman by making h4ck3d w0rd5 \n\n",
        "uwu <message>: Translate to the uwu wanguwage uwu"
    );

    let _ = poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("Text Modification Help");
            e.description("Description: Commands that modify text");
            e.field("Commands", content, false);
            e.footer(|f| {
                f.text(concat!(
                    "Putting an l in front of any command",
                    "(except h4ck and uwu) will use the last message"
                ));
                f
            });
            e
        })
    })
    .await;
}
