use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::backend_api::objects::{ActiveBotsResponse, Trade};
use crate::structs::bot::Bot as InternalBot;
use crate::structs::trade::{Trade as InternalTrade, TradeSide};

pub trait TradeConverter<'c> {
    fn to_internal_trade(&self) -> Result<InternalTrade<'c>>;
}

impl<'c> TradeConverter<'c> for Trade {
    fn to_internal_trade(&self) -> Result<InternalTrade<'c>> {
        Ok(InternalTrade {
            base_asset: self.base_asset.to_owned().into(),
            quote_asset: self.quote_asset.to_owned().into(),
            amount: Decimal::from_str(&self.quantity)?,
            price: Decimal::from_str(&self.price)?,
            timestamp: self.trade_timestamp,
            side: match self.trade_type.as_str() {
                "BUY" => TradeSide::Buy,
                "SELL" => TradeSide::Sell,
                _ => {
                    return Err(anyhow!("Invalid trade side"));
                }
            },
        })
    }
}

pub trait BotsConverter<'c> {
    fn to_internal_bots(&self) -> Result<Vec<InternalBot<'c>>>;
}

impl<'c> BotsConverter<'c> for ActiveBotsResponse {
    fn to_internal_bots(&self) -> Result<Vec<InternalBot<'c>>> {
        return Ok(self
            .data
            .bots
            .keys()
            .map(|name| InternalBot {
                name: name.clone().into(),
            })
            .collect());
    }
}
