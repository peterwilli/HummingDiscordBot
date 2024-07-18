use crate::structs::trade::Trade;
use crate::structs::trade::TradeSide;
use anyhow::anyhow;
use anyhow::Result;
use rust_decimal::prelude::*;

pub trait TradeCSVParser {
    fn from_line(line: &str) -> Result<Trade>;
}

impl<'c> TradeCSVParser for Trade<'c> {
    fn from_line(line: &str) -> Result<Trade> {
        let components = line.split(",").collect::<Vec<&str>>();
        if components.len() < 13 {
            return Err(anyhow!("Invalid line (too short)"));
        }
        let trade = Trade {
            base_asset: components[5].into(),
            quote_asset: components[6].into(),
            amount: Decimal::from_str(components[12])?.normalize(),
            price: Decimal::from_str(components[11])?.normalize(),
            side: if components[9] == "BUY" {
                TradeSide::Buy
            } else {
                TradeSide::Sell
            },
            timestamp: components[7].parse()?,
        };
        return Ok(trade);
    }
}
