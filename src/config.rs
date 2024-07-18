use std::{borrow::Cow, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BotEntry<'c> {
    pub name: Cow<'c, str>,
    /// For profit tracking
    pub base_asset: Cow<'c, str>,
    pub trades_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config<'c> {
    pub bot_token: Cow<'c, str>,
    pub stats_channel_id: u64,
    pub bots: Vec<BotEntry<'c>>,
}

impl<'c> Default for Config<'c> {
    fn default() -> Self {
        return Self {
            bot_token: "MY_TOKEN".into(),
            stats_channel_id: 39923329,
            bots: vec![BotEntry {
                name: "TheBot".into(),
                base_asset: "USDT".into(),
                trades_path: PathBuf::from_str("/tmp/trades.csv").unwrap(),
            }],
        };
    }
}
