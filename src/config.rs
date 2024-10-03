use std::{borrow::Cow, path::PathBuf};

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledStats<'c> {
    pub message: Cow<'c, str>,
    pub schedule: Cow<'c, str>,
    pub enabled: bool,
    pub channel_id: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config<'c> {
    pub bot_token: Cow<'c, str>,
    pub stats_channel_id: u64,
    pub backend_api_base_url: Url,
    pub cache_path: PathBuf,
    pub cache_strip_bot_names: bool,
    pub scheduled_chart_announcement: ScheduledStats<'c>,
}

impl<'c> Default for Config<'c> {
    fn default() -> Self {
        Self {
            bot_token: "MY_TOKEN".into(),
            stats_channel_id: 39923329,
            cache_path: PathBuf::from("/storage/mdh_discord/cache"),
            cache_strip_bot_names: true,
            backend_api_base_url: Url::parse("http://backend-api:8000").unwrap(),
            scheduled_chart_announcement: ScheduledStats {
                message: "Good morning! Here are the scheduled profits (or losses) from yesterdays operation 💸".into(),
                schedule: "0 0 9 * * *".into(),
                enabled: false,
                channel_id: 29384550,
            },
        }
    }
}
