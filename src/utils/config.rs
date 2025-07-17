use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, MessageId};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub contest_start_timestamp: i64,
    pub contest_end_timestamp: i64,
    pub feed_channel: ChannelId,
    pub permanent_leaderboard: (ChannelId, MessageId)
}
