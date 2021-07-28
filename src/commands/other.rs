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
    poise::defaults::register_slash_commands(ctx, global).await?;

    Ok(())
}
