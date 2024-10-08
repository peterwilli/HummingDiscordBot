use anyhow::{Ok, Result};
use rust_decimal::Decimal;
use std::{borrow::Cow, collections::HashMap};

use crate::backend_api::client::BackendAPIClient;

use super::{extensions::converter::TradeConverter, trade::Trade};

#[derive(Default)]
pub struct BotPNL {
    pub pct: Decimal,
    pub quote: Decimal,
}

pub struct BotController {
    pub pnl: BotPNL,
}

pub struct Bot<'c> {
    pub name: Cow<'c, str>,
    pub global_pnl: BotPNL,
    pub controllers: HashMap<String, BotController>,
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
            Some(trade) => Ok(Some(trade.to_internal_trade()?)),
            None => Ok(None),
        }
    }
}
