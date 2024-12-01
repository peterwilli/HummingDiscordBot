use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartDataEntry {
    pub timestamp: u64,
    pub balance: Decimal,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChartData {
    pub chart_data: HashMap<String, Vec<ChartDataEntry>>,
}
