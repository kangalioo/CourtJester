mod commands;
mod handlers;
mod helpers;
mod reactions;
mod structures;

use crate::{
    handlers::{
        event_handler::{LavalinkHandler, SerenityHandler},
        framework::get_framework,
    },
    helpers::{command_utils, database_helper},
    structures::{cmd_data::*, commands::*, errors::*},
};
use aspotify::{Client as Spotify, ClientCredentials};
use dashmap::DashMap;
use futures::future::AbortHandle;
use lavalink_rs::LavalinkClient;
use reqwest::Client as Reqwest;
use serenity::{
    client::bridge::gateway::GatewayIntents,
    framework::{
        standard::{CommandError, CommandResult},
        StandardFramework,
    },
    http::Http,
    model::id::{ApplicationId, GuildId},
    prelude::*,
};
use songbird::SerenityInit;
use std::{
    collections::{HashMap, HashSet},
    env,
    sync::{atomic::AtomicBool, Arc},
};

pub type Context<'a> = poise::Context<'a, Data, CommandError>;
pub type PrefixContext<'a> = poise::PrefixContext<'a, Data, CommandError>;

#[tokio::main]
async fn main() -> CommandResult {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    let creds = helpers::credentials_helper::read_creds(&args[1])?;
    let token = &creds.bot_token;

    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let pool = database_helper::obtain_db_pool(creds.db_connection).await?;
    let prefixes = database_helper::fetch_prefixes(&pool).await?;
    let voice_timer_map: DashMap<GuildId, AbortHandle> = DashMap::new();

    let lava_client = LavalinkClient::builder(bot_id)
        .set_host(creds.lavalink_host)
        .set_password(creds.lavalink_auth)
        .build(LavalinkHandler)
        .await?;

    let mut pub_creds = HashMap::new();
    pub_creds.insert("tenor".to_string(), creds.tenor_key);
    pub_creds.insert("default prefix".to_string(), creds.default_prefix.clone());

    let client_credentials = ClientCredentials {
        id: creds.spotify_client_id,
        secret: creds.spotify_client_secret,
    };

    let spotify = Spotify::new(client_credentials);

    let emergency_commands = command_utils::get_allowed_commands();

    let command_names = MASTER_GROUP
        .options
        .sub_groups
        .iter()
        .flat_map(|x| {
            x.options
                .commands
                .iter()
                .flat_map(|i| i.options.names.iter().map(ToString::to_string))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<String>>();

    let reqwest_client = Reqwest::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0")
        .build()?;

    let mut client = Client::builder(&token)
        .application_id(bot_id.0)
        .framework(get_framework(bot_id, owners))
        .event_handler(SerenityHandler {
            run_loop: AtomicBool::new(true),
        })
        .intents({
            let mut intents = GatewayIntents::all();
            intents.remove(GatewayIntents::DIRECT_MESSAGES);
            intents.remove(GatewayIntents::DIRECT_MESSAGE_REACTIONS);
            intents.remove(GatewayIntents::DIRECT_MESSAGE_TYPING);
            intents
        })
        .register_songbird()
        .await
        .expect("Err creating client");

    let data = {
        let spotify_client = Arc::new(spotify);
        let data_struct = Data {
            connection_pool: pool.clone(),
            shard_manager_container: Arc::clone(&client.shard_manager),
            lavalink: lava_client.clone(),
            voice_timer_map: Arc::new(voice_timer_map.clone()),
            prefix_map: Arc::new(prefixes.clone()),
            command_name_map: Arc::new(command_names.clone()),
            reqwest_client: reqwest_client.clone(),
            pub_creds: Arc::new(pub_creds.clone()),
            emergency_commands: Arc::new(emergency_commands.clone()),
            bot_id,
            spotify_client: Arc::clone(&spotify_client),
            reaction_image_cache: Arc::new(DashMap::new()),
        };

        // Insert all structures into ctx data
        let mut data = client.data.write().await;

        data.insert::<ConnectionPool>(pool.clone());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<Lavalink>(lava_client);
        data.insert::<VoiceTimerMap>(Arc::new(voice_timer_map));
        data.insert::<PrefixMap>(Arc::new(prefixes));
        data.insert::<CommandNameMap>(Arc::new(command_names));
        data.insert::<ReqwestClient>(reqwest_client);
        data.insert::<PubCreds>(Arc::new(pub_creds));
        data.insert::<EmergencyCommands>(Arc::new(emergency_commands));
        data.insert::<BotId>(bot_id);
        data.insert::<SpotifyClient>(Arc::clone(&spotify_client));
        data.insert::<ReactionImageCache>(Arc::new(DashMap::new()));

        data_struct
    };

    let mut options = poise::FrameworkOptions::<Data, CommandError> {
        prefix_options: poise::PrefixFrameworkOptions {
            edit_tracker: Some(poise::EditTracker::for_timespan(
                std::time::Duration::from_secs(3600),
            )),
            ..Default::default()
        },
        ..Default::default()
    };

    options.command(commands::ciphers::b64decode(), |f| f);
    options.command(commands::ciphers::b64encode(), |f| f);
    options.command(commands::images::disgust(), |f| f);
    options.command(commands::images::cry(), |f| f);
    options.command(commands::images::cringe(), |f| f);
    options.command(commands::images::gifsearch(), |f| f);
    options.command(commands::other::register(), |f| f);
    options.command(commands::other::ping(), |f| f);
    options.command(commands::support::help(), |f| f);
    options.command(commands::support::support(), |f| f);
    options.command(commands::support::info(), |f| f);
    options.command(commands::textchannel_send::nice(), |f| f);
    options.command(commands::textchannel_send::bruh(), |f| f);
    options.command(commands::textchannel_send::quote(), |f| f);
    options.command(commands::textchannel_send::vibecheck(), |f| f);
    options.command(commands::textmod::mock(), |f| f);
    options.command(commands::textmod::inv(), |f| f);
    options.command(commands::textmod::upp(), |f| f);
    options.command(commands::textmod::low(), |f| f);
    options.command(commands::textmod::space(), |f| f);
    options.command(commands::textmod::biggspace(), |f| f);
    options.command(commands::textmod::h4ck(), |f| f);
    options.command(commands::textmod::uwu(), |f| f);
    options.command(commands::textmod::mockl(), |f| f);
    options.command(commands::textmod::invl(), |f| f);
    options.command(commands::textmod::uppl(), |f| f);
    options.command(commands::textmod::lowl(), |f| f);
    options.command(commands::textmod::spacel(), |f| f);
    options.command(commands::textmod::biggspacel(), |f| f);
    options.command(commands::utility::avatar(), |f| f);
    // options.command(commands::utility::kang(), |f| f);
    options.command(commands::utility::emoji_info(), |f| f);
    options.command(commands::utility::spoiler(), |f| f);

    let prefix_and_slash_command = (
        commands::images::prefix_hug().0,
        commands::images::slash_hug().1,
    );
    options.command(prefix_and_slash_command, |f| f);

    let prefix_and_slash_command = (
        commands::images::prefix_pat().0,
        commands::images::slash_pat().1,
    );
    options.command(prefix_and_slash_command, |f| f);

    let prefix_and_slash_command = (
        commands::images::prefix_kiss().0,
        commands::images::slash_kiss().1,
    );

    options.command(prefix_and_slash_command, |f| f);
    let prefix_and_slash_command = (
        commands::images::prefix_slap().0,
        commands::images::slash_slap().1,
    );

    options.command(prefix_and_slash_command, |f| f);

    let framework = poise::Framework::new(
        creds.default_prefix,
        ApplicationId(bot_id.0),
        |_, _, _| Box::pin(async { Ok(data) }),
        options,
    );

    // Start up the bot! If there's an error, let the user know
    if let Err(why) = tokio::try_join!(
        client.start_autosharded(),
        framework.start(Client::builder(&token).framework(StandardFramework::new()))
    ) {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}
