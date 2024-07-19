use anyhow::{Ok, Result};
use std::{borrow::Cow, path::PathBuf};

use crate::backend_api::client::BackendAPIClient;

use super::{extensions::converter::TradeConverter, trade::Trade};

pub struct Bot<'c> {
    pub name: Cow<'c, str>,
}

impl<'c> Bot<'c> {
    pub async fn get_trades(&self, client: &BackendAPIClient) -> Result<Vec<Trade<'c>>> {
        let trades = client.get_trades(&self.name).await?;
        let converted_trades = trades
            .iter()
            .map(|t| t.to_internal_trade().unwrap())
            .collect::<Vec<Trade<'c>>>();
        Ok(converted_trades)
    }

    pub async fn get_latest_trade(&self, client: &BackendAPIClient) -> Result<Option<Trade<'c>>> {
        let trade = client.get_latest_trade(&self.name).await?;
        match trade {
            Some(trade) => {
                return Ok(Some(trade.to_internal_trade()?));
            }
            None => {
                return Ok(None);
            }
        }
    }
}
