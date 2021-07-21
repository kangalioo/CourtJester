use serde::{Deserialize, Serialize};
use serenity::framework::standard::CommandResult;
use std::fs;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub bot_token: String,
    pub default_prefix: String,
    #[serde(skip)]
    pub db_connection: String,
    pub lavalink_host: String,
    pub lavalink_auth: String,
    pub tenor_key: String,
    pub spotify_client_id: String,
    pub spotify_client_secret: String,
}

pub fn read_creds(path: &str, db_connection: String) -> CommandResult<Credentials> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut info: Credentials = serde_json::from_reader(reader).unwrap();
    info.db_connection = db_connection;

    Ok(info)
}
