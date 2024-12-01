use anyhow::{anyhow, Result};

use rust_decimal::Decimal;

use std::collections::HashMap;
use std::str::FromStr;

use crate::backend_api::objects::{Account, ActiveBotsResponse, Trade};
use crate::structs::bot::{Bot as InternalBot, BotController, BotPNL};
use crate::structs::bot_balance::{BotBalance, BotBalanceEntry};
use crate::structs::trade::{Trade as InternalTrade, TradeSide};

pub trait AccountStateConverter {
    fn to_bot_balance(&self) -> BotBalance;
}

impl AccountStateConverter for Account {
    fn to_bot_balance(&self) -> BotBalance {
        let mut result = BotBalance::default();
        for (credentials, exchange_to_tokens) in self.iter() {
            let mut coins = HashMap::new();
            for (exchange, tokens) in exchange_to_tokens.iter() {
                coins.insert(
                    exchange.clone(),
                    tokens
                        .iter()
                        .map(|t| BotBalanceEntry {
                            coin: t.token.clone(),
                            amount: t.value,
                        })
                        .collect(),
                );
            }
            result.accounts.insert(credentials.clone(), coins);
        }
        result
    }
}

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
