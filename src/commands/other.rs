use crate::{Context, PrefixContext};
use serenity::framework::standard::CommandResult;

/// Prints "Pong!". Quick and easy way to see if the bot's online.
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> CommandResult {
    poise::say_reply(ctx, "Pong!".into()).await?;

    Ok(())
}

/// Register slash commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
#[poise::command(hide_in_help)]
pub async fn register(ctx: PrefixContext<'_>, #[flag] global: bool) -> CommandResult {
    let guild = ctx
        .msg
        .guild(ctx.discord)
        .await
        .ok_or("Must be called in guild")?;

    if ctx.msg.author.id != guild.owner_id {
        return Err("Can only be used by server owner".into());
    }

    let commands = &ctx.framework.options().slash_options.commands;
    poise::say_prefix_reply(ctx, format!("Registering {} commands...", commands.len())).await?;
    for cmd in commands {
        if global {
            cmd.create_global(&ctx.discord.http).await?;
        } else {
            cmd.create_in_guild(&ctx.discord.http, guild.id).await?;
        }
    }
    poise::say_prefix_reply(ctx, "Done!".to_owned()).await?;
    Ok(())
}
