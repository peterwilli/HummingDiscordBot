use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct CloseTypeCounts {
    #[serde(rename = "CloseType.EARLY_STOP")]
    pub early_stop: u32,
    #[serde(rename = "CloseType.TIME_LIMIT")]
    pub time_limit: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Performance {
    pub status: String,
    pub realized_pnl_quote: f64,
    pub unrealized_pnl_quote: f64,
    pub unrealized_pnl_pct: f64,
    pub realized_pnl_pct: f64,
    pub global_pnl_quote: f64,
    pub global_pnl_pct: f64,
    pub volume_traded: f64,
    pub open_order_volume: f64,
    pub inventory_imbalance: f64,
    pub close_type_counts: CloseTypeCounts,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BotPerformance {
    pub status: String,
    pub performance: Performance,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    #[serde(flatten)]
    pub bots: HashMap<String, BotPerformance>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ActiveBotsResponse {
    pub status: String,
    pub data: Data,
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
