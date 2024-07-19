use anyhow::{anyhow, Result};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

use crate::backend_api::objects::{ActiveBotsResponse, Trade};
use crate::structs::bot::{Bot as InternalBot, BotController, BotPNL};
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
    fn to_internal_bots(&self) -> Vec<InternalBot<'c>>;
}

impl<'c> BotsConverter<'c> for ActiveBotsResponse {
    fn to_internal_bots(&self) -> Vec<InternalBot<'c>> {
        return self
            .data
            .iter()
            .map(|(name, bot)| {
                let mut global_pnl = BotPNL::default();
                let controllers = bot
                    .performance
                    .iter()
                    .map(|(n, c)| {
                        global_pnl.quote += c.performance.global_pnl_quote;
                        global_pnl.quote += c.performance.global_pnl_quote;

                        (
                            n.clone(),
                            BotController {
                                pnl: BotPNL {
                                    pct: c.performance.global_pnl_pct,
                                    quote: c.performance.global_pnl_quote,
                                },
                            },
                        )
                    })
                    .collect();
                InternalBot {
                    name: name.clone().into(),
                    global_pnl,
                    controllers,
                }
            })
            .collect();
    }
}
