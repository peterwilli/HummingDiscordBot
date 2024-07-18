use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartDataEntry {
    pub timestamp: u64,
    pub profit: Decimal,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartData<'c> {
    pub bot_name: Cow<'c, str>,
    pub base_asset: Cow<'c, str>,
    pub chart_data: Vec<ChartDataEntry>,
}
