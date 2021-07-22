use serenity::{builder::CreateEmbed, framework::standard::CommandResult};

use crate::{
    commands::{
        ciphers::*, config::*, images::*, japan::*, music::*, starboard::*, textchannel_send::*,
        textmod::*, utility::*,
    },
    helpers::{botinfo::*, command_utils, voice_utils::*},
};

/// Show information about the bot's commands
#[poise::command(slash_command)]
pub async fn help(
    ctx: crate::Context<'_>,
    #[description = "Specific command to show help about"] subcommand: Option<String>,
) -> CommandResult {
    let subcommand = match subcommand {
        Some(x) => x,
        None => {
            match ctx {
                crate::Context::Prefix(c) if command_utils::check_mention_prefix(c.msg) => {
                    emergency_help_message(ctx).await
                }
                _ => default_help_message(ctx).await,
            }

            return Ok(());
        }
    };

    match subcommand.as_str() {
        "prefix" => prefix_help(ctx.discord(), ctx.channel_id()).await,
        "command" => command_help(ctx.discord(), ctx.channel_id()).await,
        "starboard" => starboard_help(ctx.discord(), ctx.channel_id()).await,
        "utilities" => utility_help(ctx.discord(), ctx.channel_id()).await,
        "senders" => sender_help(ctx.discord(), ctx.channel_id()).await,
        "ciphers" => cipher_help(ctx.discord(), ctx.channel_id()).await,
        "text" => textmod_help(ctx.discord(), ctx.channel_id()).await,
        "voice" => voice_help(ctx.discord(), ctx.channel_id()).await,
        "music" => music_help(ctx.discord(), ctx.channel_id()).await,
        "images" => image_help(ctx.discord(), ctx.channel_id()).await,
        "japan" => japan_help(ctx.discord(), ctx.channel_id()).await,
        _ => {}
    }

    Ok(())
}

async fn emergency_help_message(ctx: crate::Context<'_>) {
    let content = concat!(
        "prefix (characters): Sets the server's bot prefix \n\n",
        "resetprefix: Reset's the server's prefix back to the default one"
    );

    let _ = poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("CourtJester Emergency Help");
            e.description("You should only use this if you mess up your prefix!");
            e.field("Commands", content, false);
            e
        })
    })
    .await;
}

async fn default_help_message(ctx: crate::Context<'_>) {
    let categories = concat!(
        "prefix \n",
        "command \n",
        "starboard \n",
        "utilities \n",
        "senders \n",
        "ciphers \n",
        "text \n",
        "voice \n",
        "music \n",
        "images \n",
        "japan \n"
    );

    let _ = poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("CourtJester Help");
            e.description(concat!(
                "Help for the CourtJester Discord bot \n",
                "Command parameters: <> is required and () is optional \n",
                "Please use `help <subcategory>` to see that category's help"
            ));
            e.field("Subcategories", format!("```\n{}```", categories), false);
            e.footer(|f| {
                f.text("Use the support command for any further help!");
                f
            });
            e
        })
    })
    .await;
}

/// Support information for bot users
#[poise::command(slash_command)]
pub async fn support(ctx: crate::Context<'_>) -> CommandResult {
    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("CourtJester Support");
            e.description("Need more help?");
            e.field("Support Server", "https://discord.gg/pswt7by", false);
            e.field(
                "Github repository",
                "https://github.com/bdashore3/courtjester",
                false,
            );
            e.field("kingbri's twitter", "https://twitter.com/kingbri1st", false);
            e.footer(|f| {
                f.text("Created with ❤️ by kingbri#6666");
                f
            })
        })
    })
    .await?;

    Ok(())
}

/// Information about the bot
#[poise::command(slash_command)]
pub async fn info(ctx: crate::Context<'_>) -> CommandResult {
    let mut eb = CreateEmbed::default();

    let guild_count = ctx.discord().cache.guilds().await.len();
    let channel_count = ctx.discord().cache.guild_channel_count().await;
    let user_count = ctx.discord().cache.user_count().await;

    let guild_name = if guild_count < 2 { "guild" } else { "guilds" };

    let last_commit = get_last_commit(ctx).await?;
    let sys_info = get_system_info(ctx).await?;

    let mut story_string = String::new();
    story_string.push_str(&format!(
        "Currently running on commit [{}]({}) \n",
        &last_commit.sha[..7],
        last_commit.html_url
    ));
    story_string.push_str(&format!("Inside `{}` {} \n", guild_count, guild_name));
    story_string.push_str(&format!("With `{}` total channels \n", channel_count));
    story_string.push_str(&format!("Along with `{}` faithful users \n", user_count));
    story_string.push_str(&format!(
        "Consuming `{:.3} MB` of memory \n",
        sys_info.memory
    ));
    story_string.push_str(&format!("With a latency of `{}`", sys_info.shard_latency));

    eb.title("CourtJester is");
    eb.color(0xfda50f);
    eb.description(story_string);

    poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.0 = eb.0;
            e
        })
    })
    .await?;

    Ok(())
}
