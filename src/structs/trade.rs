use rust_decimal::prelude::*;
use std::borrow::Cow;
use strum::{Display, EnumString, IntoStaticStr};

#[derive(EnumString, Display, IntoStaticStr, Debug, Clone)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Trade<'c> {
    pub base_asset: Cow<'c, str>,
    pub quote_asset: Cow<'c, str>,
    pub amount: Decimal,
    pub price: Decimal,
    pub timestamp: u64,
    pub side: TradeSide,
}
