use std::str::FromStr;
use test_log::test;
use url::Url;

use crate::backend_api::client::BackendAPIClient;
use crate::structs::extensions::converter::TradeConverter;

#[test(tokio::test)]
async fn test_trade_api() {
    let client = BackendAPIClient::new(Url::from_str("http://localhost:8084").unwrap());
    let trade = client
        .get_latest_trade("hummingbot-hyperliquid_test-2024.07.17_09.46")
        .await
        .unwrap()
        .unwrap();
    println!("{:?}", trade);
    println!("{:?}", trade.to_internal_trade());
}

#[test(tokio::test)]
async fn test_bots_api() {
    let client = BackendAPIClient::new(Url::from_str("http://localhost:8084").unwrap());
    let bots = client.get_bots().await.unwrap();
    println!("{:?}", bots);
}
