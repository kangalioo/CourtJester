use serenity::{
    client::Context,
    framework::standard::{Args, CommandResult, Delimiter},
    model::prelude::*,
    utils::parse_channel,
};
use sqlx::PgPool;
use std::time::Duration;

#[poise::command(required_permissions = "MANAGE_MESSAGES")]
pub async fn starboard(ctx: crate::PrefixContext<'_>) -> CommandResult {
    starboard_help(crate::Context::Prefix(ctx)).await;

    Ok(())
}

#[poise::command]
pub async fn threshold(ctx: crate::PrefixContext<'_>, new_threshold: u32) -> CommandResult {
    let (ctx, msg, data) = (ctx.discord, ctx.msg, ctx.data);

    sqlx::query!(
        "UPDATE guild_info SET starboard_threshold = $1 WHERE guild_id = $2",
        new_threshold as i32,
        msg.guild_id.unwrap().0 as i64
    )
    .execute(&data.connection_pool)
    .await?;

    msg.channel_id
        .say(ctx, "New threshold sucessfully set!")
        .await?;

    Ok(())
}

#[poise::command]
pub async fn channel(ctx: crate::PrefixContext<'_>, new_channel: ChannelId) -> CommandResult {
    let (ctx, msg, data) = (ctx.discord, ctx.msg, ctx.data);

    sqlx::query!(
        "INSERT INTO text_channels VALUES($1, null, null, $2)
                ON CONFLICT (guild_id)
                DO UPDATE SET quote_id = $2",
        msg.guild_id.unwrap().0 as i64,
        new_channel.0 as i64
    )
    .execute(&data.connection_pool)
    .await?;

    msg.channel_id
        .say(ctx, "New starboard channel sucessfully set!")
        .await?;

    Ok(())
}

#[poise::command]
pub async fn deactivate(ctx: crate::PrefixContext<'_>) -> CommandResult {
    let (ctx, msg, data) = (ctx.discord, ctx.msg, ctx.data);

    let author_id = msg.author.id;
    let channel_id = msg.channel_id;

    let sent_message = msg
        .channel_id
        .say(
            ctx,
            "Removing the starboard re-enables quoting! You want to do this?",
        )
        .await?;
    sent_message
        .react(ctx, ReactionType::Unicode(String::from("✅")))
        .await?;
    sent_message
        .react(ctx, ReactionType::Unicode(String::from("❌")))
        .await?;

    let reaction_action = sent_message
        .await_reaction(ctx)
        .filter(move |reaction| {
            reaction.user_id == Some(author_id) && reaction.channel_id == channel_id
        })
        .timeout(Duration::from_secs(120))
        .await;

    match reaction_action {
        Some(action) => {
            let reaction = action.as_inner_ref();
            let reaction_emoji = &reaction.emoji.as_data();

            if reaction_emoji == "✅" {
                sqlx::query!(
                    "UPDATE guild_info SET starboard_threshold = null WHERE guild_id = $1",
                    msg.guild_id.unwrap().0 as i64
                )
                .execute(&data.connection_pool)
                .await?;

                sqlx::query!(
                    "UPDATE text_channels SET quote_id = null WHERE guild_id = $1",
                    msg.guild_id.unwrap().0 as i64
                )
                .execute(&data.connection_pool)
                .await?;

                msg.channel_id
                    .say(ctx, "The starboard has been deactivated")
                    .await?;
            } else if reaction_emoji == "❌" {
                msg.channel_id.say(ctx, "Aborting...").await?;
            } else {
                msg.channel_id
                    .say(ctx, "That's not a valid emoji! Aborting...")
                    .await?;
            }
        }
        None => {
            msg.channel_id.say(ctx, "Timed out").await?;
        }
    }

    Ok(())
}

#[poise::command]
pub async fn wizard(ctx: crate::PrefixContext<'_>) -> CommandResult {
    let (ctx, msg, data) = (ctx.discord, ctx.msg, ctx.data);

    let intro_string = concat!(
        "Welcome to starboard configuration \n",
        "Reacting with ✅ will disable quoting on your guild!"
    );

    let author_id = msg.author.id;
    let channel_id = msg.channel_id;

    let sent_message = msg.channel_id.say(ctx, intro_string).await?;
    sent_message
        .react(ctx, ReactionType::Unicode(String::from("✅")))
        .await?;
    sent_message
        .react(ctx, ReactionType::Unicode(String::from("❌")))
        .await?;

    let reaction_action = sent_message
        .await_reaction(ctx)
        .timeout(Duration::from_secs(120))
        .filter(move |reaction| {
            reaction.user_id == Some(author_id) && reaction.channel_id == channel_id
        })
        .await;

    match reaction_action {
        Some(action) => {
            let reaction = action.as_inner_ref();

            let reaction_emoji = &reaction.emoji.as_data();

            if reaction_emoji == "✅" {
                starboard_wizard_threshold(ctx, msg, &data.connection_pool).await?
            } else if reaction_emoji == "❌" {
                msg.channel_id.say(ctx, "Aborting...").await?;
            } else {
                msg.channel_id
                    .say(ctx, "That's not a valid emoji! Aborting...")
                    .await?;
            }
        }
        None => {
            msg.channel_id.say(ctx, "Timed out").await?;
        }
    }

    Ok(())
}

async fn starboard_wizard_threshold(ctx: &Context, msg: &Message, pool: &PgPool) -> CommandResult {
    msg.channel_id
        .say(
            ctx,
            "Sounds good! Please enter a number greater than 0 for the starboard threshold!",
        )
        .await?;

    let channel_id = msg.channel_id;

    loop {
        let threshold_message = msg
            .author
            .await_reply(ctx)
            .timeout(Duration::from_secs(120))
            .filter(move |given_msg| given_msg.channel_id == channel_id)
            .await;

        match threshold_message {
            Some(message) => {
                match message.content.parse::<u32>() {
                    Ok(threshold) => {
                        if threshold > 0 {
                            sqlx::query!("UPDATE guild_info SET starboard_threshold = $1 WHERE guild_id = $2", 
                                    threshold as i32, msg.guild_id.unwrap().0 as i64)
                                .execute(pool).await?;

                            break;
                        } else {
                            msg.channel_id
                                .say(ctx, "Please enter an integer greater than 0!")
                                .await?;
                        }
                    }
                    Err(_) => {
                        msg.channel_id
                            .say(ctx, "Please enter an integer greater than 0!")
                            .await?;
                    }
                }
            }
            None => {
                msg.channel_id.say(ctx, "Timed out").await?;

                return Ok(());
            }
        }
    }

    starboard_wizard_channel(ctx, msg, pool).await?;

    Ok(())
}

async fn starboard_wizard_channel(ctx: &Context, msg: &Message, pool: &PgPool) -> CommandResult {
    let mut channel_check = false;

    let row_check = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM text_channels WHERE guild_id = $1)",
        msg.guild_id.unwrap().0 as i64
    )
    .fetch_one(pool)
    .await?;

    if row_check.exists.unwrap() {
        let query = sqlx::query!(
            "SELECT quote_id FROM text_channels WHERE guild_id = $1",
            msg.guild_id.unwrap().0 as i64
        )
        .fetch_one(pool)
        .await?;

        if query.quote_id.is_some() {
            channel_check = true;
        } else {
            channel_check = false;
        }
    };

    if channel_check {
        let send_string = concat!(
            "You already have a channel set up for quotes! \nIf you want to change it, run `starboard channel <mention>` \n",
            "Enjoy your new starboard!");
        msg.channel_id.say(ctx, send_string).await?;
    } else {
        msg.channel_id
            .say(
                ctx,
                "Now please mention the channel you want messages sent to!",
            )
            .await?;
        let channel_id = msg.channel_id;

        loop {
            let channel_message = msg
                .author
                .await_reply(ctx)
                .timeout(Duration::from_secs(120))
                .filter(move |given_msg| given_msg.channel_id == channel_id)
                .await;

            match channel_message {
                Some(message) => {
                    let args = Args::new(&message.content, &[Delimiter::Single(' ')]);
                    let given_id = args.parse::<String>().unwrap();

                    match parse_channel(given_id) {
                        Some(channel_id) => {
                            sqlx::query!(
                                "INSERT INTO text_channels VALUES($1, null, null, $2)
                                        ON CONFLICT (guild_id)
                                        DO UPDATE SET quote_id = $2",
                                msg.guild_id.unwrap().0 as i64,
                                channel_id as i64
                            )
                            .execute(pool)
                            .await?;

                            msg.channel_id.say(ctx, "Enjoy your new starboard!").await?;
                            break;
                        }
                        None => {
                            msg.channel_id
                                .say(ctx, "Please mention a channel in this guild!")
                                .await?;
                        }
                    }
                }
                None => {
                    msg.channel_id.say(ctx, "Timed out").await?;

                    return Ok(());
                }
            }
        }
    }

    Ok(())
}

pub async fn starboard_help(ctx: crate::Context<'_>) {
    let content = concat!(
        "wizard: Easy way to setup the starboard \n\n",
        "threshold: Sets the threshold for a message to appear \n\n",
        "channel: Sets the channel where starboard embeds are sent \n\n",
        "deactivate: Deactivates the starboard and re-enables quoting"
    );

    let _ = poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title("Starboard Help");
            e.description("Description: admin commands for starboarding in a discord server");
            e.field("Commands", content, false);
            e.footer(|f| {
                f.text("Enabling the starboard will disable the quote command!");
                f
            });
            e
        })
    })
    .await;
}
