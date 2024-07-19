use std::{borrow::Cow};

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config<'c> {
    pub bot_token: Cow<'c, str>,
    pub stats_channel_id: u64,
    pub backend_api_base_url: Url,
}

impl<'c> Default for Config<'c> {
    fn default() -> Self {
        Self {
            bot_token: "MY_TOKEN".into(),
            stats_channel_id: 39923329,
            backend_api_base_url: Url::parse("http://localhost:8084").unwrap(),
        }
    }
}
