use aspotify::Client as Spotify;
use dashmap::DashMap;
use futures::future::AbortHandle;
use lavalink_rs::LavalinkClient;
use reqwest::Client as Reqwest;
use serenity::{
    client::bridge::gateway::ShardManager,
    model::id::{GuildId, UserId},
    prelude::{Mutex, TypeMapKey},
};
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};

// All command context data structures
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}

pub struct Lavalink;

impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}

pub struct VoiceTimerMap;

impl TypeMapKey for VoiceTimerMap {
    type Value = Arc<DashMap<GuildId, AbortHandle>>;
}

pub struct CommandNameMap;

impl TypeMapKey for CommandNameMap {
    type Value = Arc<Vec<String>>;
}

pub struct ReqwestClient;

impl TypeMapKey for ReqwestClient {
    type Value = Reqwest;
}

pub struct PubCreds;

impl TypeMapKey for PubCreds {
    type Value = Arc<HashMap<String, String>>;
}

pub struct BotId;

impl TypeMapKey for BotId {
    type Value = UserId;
}

pub struct PrefixMap;

impl TypeMapKey for PrefixMap {
    type Value = Arc<DashMap<GuildId, String>>;
}

pub struct EmergencyCommands;

impl TypeMapKey for EmergencyCommands {
    type Value = Arc<Vec<String>>;
}

pub struct SpotifyClient;

impl TypeMapKey for SpotifyClient {
    type Value = Arc<Spotify>;
}

pub struct ReactionImageCache;

impl TypeMapKey for ReactionImageCache {
    type Value = Arc<DashMap<(GuildId, String), String>>;
}

pub struct Data {
    pub shard_manager_container: Arc<Mutex<ShardManager>>,
    pub connection_pool: PgPool,
    pub lavalink: LavalinkClient,
    pub voice_timer_map: Arc<DashMap<GuildId, AbortHandle>>,
    pub command_name_map: Arc<Vec<String>>,
    pub reqwest_client: Reqwest,
    pub pub_creds: Arc<HashMap<String, String>>,
    pub bot_id: UserId,
    pub prefix_map: Arc<DashMap<GuildId, String>>,
    pub emergency_commands: Arc<Vec<String>>,
    pub spotify_client: Arc<Spotify>,
    pub reaction_image_cache: Arc<DashMap<(GuildId, String), String>>,
}
