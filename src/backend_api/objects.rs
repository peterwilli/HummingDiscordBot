use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct CloseTypeCounts {
    #[serde(rename = "CloseType.EARLY_STOP", default)]
    pub early_stop: u64,
    #[serde(rename = "CloseType.TIME_LIMIT", default)]
    pub time_limit: u64,
    #[serde(rename = "CloseType.TRAILING_STOP", default)]
    pub trailing_stop: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Performance {
    pub realized_pnl_quote: Decimal,
    pub unrealized_pnl_quote: Decimal,
    pub unrealized_pnl_pct: Decimal,
    pub realized_pnl_pct: Decimal,
    pub global_pnl_quote: Decimal,
    pub global_pnl_pct: Decimal,
    pub volume_traded: Decimal,
    pub open_order_volume: Decimal,
    pub inventory_imbalance: Decimal,
    pub close_type_counts: CloseTypeCounts,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Controller {
    pub status: String,
    pub performance: Performance,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Bot {
    pub status: String,
    pub performance: HashMap<String, Controller>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ActiveBotsResponse {
    pub status: String,
    pub data: HashMap<String, Bot>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Trade {
    pub market: String,
    pub trade_id: String,
    pub price: String,
    pub quantity: String,
    pub symbol: String,
    pub trade_timestamp: u64,
    pub trade_type: String,
    pub base_asset: String,
    pub quote_asset: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TradesResponseInner {
    pub status: u16,
    pub msg: String,
    pub trades: Vec<Trade>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TradesResponse {
    pub status: String,
    pub response: TradesResponseInner,
}
