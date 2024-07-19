use anyhow::Result;
use reqwest::Client;
use url::Url;

use super::objects::{ActiveBotsResponse, Trade, TradesResponse};

pub struct BackendAPIClient {
    base_url: Url,
    client: Client,
}

impl BackendAPIClient {
    pub fn new(base_url: Url) -> BackendAPIClient {
        BackendAPIClient {
            base_url,
            client: Client::new(),
        }
    }

    pub async fn get_bots(&self) -> Result<ActiveBotsResponse> {
        let json = self
            .client
            .get(
                self.base_url
                    .join("get-active-bots-status")
                    .unwrap()
                    .as_str(),
            )
            .send()
            .await?
            .json::<ActiveBotsResponse>()
            .await?;
        return Ok(json);
    }

    pub async fn get_latest_trade(&self, bot_name: &str) -> Result<Option<Trade>> {
        let trades = self.get_trades(bot_name).await?;
        return Ok(trades.last().cloned());
    }

    pub async fn get_trades(&self, bot_name: &str) -> Result<Vec<Trade>> {
        let json = self
            .client
            .get(
                self.base_url
                    .join(&format!("get-bot-history/{}", bot_name))
                    .unwrap()
                    .as_str(),
            )
            .send()
            .await?
            .json::<TradesResponse>()
            .await?;

        return Ok(json.response.trades);
    }
}
