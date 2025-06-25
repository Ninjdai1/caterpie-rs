use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub contest_start_timestamp: i64,
    pub feed_channel: ChannelId
}
