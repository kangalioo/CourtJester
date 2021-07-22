use serenity::framework::standard::macros::group;

use crate::{
    commands::{
        config::*, japan::*, music::REMOVE_COMMAND, music::*, starboard::*, support::*,
        textchannel_send::*, textmod::*, utility::*,
    },
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
#[commands(mock, inv, upp, low, space, biggspace, h4ck, uwu)]
pub struct Text;

#[group]
#[help_available(false)]
#[commands(mockl, invl, uppl, lowl, spacel, biggspacel)]
pub struct TextLast;

#[group("Senders")]
#[description = "Commands that send certain messages to channels"]
#[commands(nice, bruh, quote, vibecheck)]
pub struct TextChannelSend;

#[group("Bot Configuration")]
#[description = "Admin/Moderator commands that configure the bot"]
#[commands(prefix, command, resetprefix)]
pub struct Config;

#[group("Support")]
#[description = "Support commands for the bot"]
#[commands(help, support, info)]
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
#[commands(avatar, kang, emoji_info, spoiler)]
pub struct Utility;
