use std::env;

use anyhow::anyhow;
use anyhow::Result;
use reqwest::Client;
use reqwest::RequestBuilder;
use url::Url;

use super::objects::Account;
use super::objects::{ActiveBotsResponse, Trade, TradesResponse};

pub struct BackendAPIClient {
    base_url: Url,
    client: Client,
}

fn add_basic_auth(rb: RequestBuilder) -> RequestBuilder {
    if let (Ok(username), Ok(password)) = (
        env::var("BACKEND_API_USERNAME"),
        env::var("BACKEND_API_PASSWORD"),
    ) {
        rb.basic_auth(username, Some(password))
    } else {
        rb
    }
}

impl BackendAPIClient {
    pub fn new(base_url: Url) -> BackendAPIClient {
        BackendAPIClient {
            base_url,
            client: Client::new(),
        }
    }

    pub async fn get_bots(&self) -> Result<ActiveBotsResponse> {
        let url = self.base_url.join("get-active-bots-status").unwrap();
        let resp = add_basic_auth(self.client.get(url.as_str()))
            .send()
            .await?
            .text()
            .await?;
        match serde_json::from_str::<ActiveBotsResponse>(&resp) {
            Ok(json) => Ok(json),
            Err(e) => Err(anyhow!(
                "Failed parsing response for {}. Body: {}\nError: {}",
                url,
                resp,
                e
            )),
        }
    }

    pub async fn get_latest_trade(&self, bot_name: &str) -> Result<Option<Trade>> {
        let trades = self.get_trades(bot_name).await?;
        return Ok(trades.last().cloned());
    }

    pub async fn get_account_state(&self) -> Result<Account> {
        let url = self.base_url.join("accounts-state").unwrap();
        let resp = add_basic_auth(self.client.get(url.as_str()))
            .send()
            .await?
            .text()
            .await?;

        match serde_json::from_str::<Account>(&resp) {
            Ok(json) => Ok(json),
            Err(e) => Err(anyhow!(
                "Failed parsing response for {}. Body: {}\nError: {}",
                url,
                resp,
                e
            )),
        }
    }

    pub async fn get_trades(&self, bot_name: &str) -> Result<Vec<Trade>> {
        let url = self
            .base_url
            .join(&format!("get-bot-history/{}", bot_name))
            .unwrap();
        let resp = add_basic_auth(self.client.get(url.as_str()))
            .send()
            .await?
            .text()
            .await?;

        match serde_json::from_str::<TradesResponse>(&resp) {
            Ok(json) => Ok(json.response.trades),
            Err(e) => Err(anyhow!(
                "Failed parsing response for {}. Body: {}\nError: {}",
                url,
                resp,
                e
            )),
        }
    }
}
