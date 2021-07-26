use serenity::framework::standard::macros::group;

use crate::{
    commands::{config::*, japan::*, music::REMOVE_COMMAND, music::*, starboard::*},
    helpers::voice_utils::*,
};

// All command groups
#[group("Master")]
#[sub_groups(
    General,
    Text,
    TextLast,
    TextChannelSend,
    Config,
    Support,
    Starboard,
    Voice,
    Images,
    Music
)]
pub struct Master;

#[group]
#[help_available(false)]
pub struct General;

#[group("Text Modification")]
#[description = "Commands than modify text. \n
Append l in the command to use the last message \n
Example: `mockl` mocks the last message"]
pub struct Text;

#[group]
#[help_available(false)]
pub struct TextLast;

#[group("Senders")]
#[description = "Commands that send certain messages to channels"]
pub struct TextChannelSend;

#[group("Bot Configuration")]
#[description = "Admin/Moderator commands that configure the bot"]
#[commands(prefix, command, resetprefix)]
pub struct Config;

#[group("Support")]
#[description = "Support commands for the bot"]
pub struct Support;

#[group("Starboard")]
#[description = "Starboard admin commands"]
#[commands(starboard)]
pub struct Starboard;

#[group("Voice")]
#[description = "Commands used for voice chat"]
#[commands(summon, disconnect)]
pub struct Voice;

#[group("Music")]
#[description = "Commands used to play music"]
#[commands(play, pause, resume, stop, skip, queue, clear, remove, seek)]
pub struct Music;

#[group("Images")]
#[description = "Commands for fetching/sending images"]
pub struct Images;

#[group("Japan")]
#[description("Commands for anime/manga")]
#[commands(anime, manga)]
pub struct Japan;

#[group("Utility")]
#[description("Server utility commands")]
pub struct Utility;
