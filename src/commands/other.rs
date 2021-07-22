use crate::Context;
use serenity::framework::standard::CommandResult;

/// Prints "Pong!". Quick and easy way to see if the bot's online.
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> CommandResult {
    poise::say_reply(ctx, "Pong!".into()).await?;

    Ok(())
}
