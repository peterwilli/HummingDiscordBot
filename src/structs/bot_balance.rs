use std::collections::HashMap;

use rust_decimal::{prelude::Zero, Decimal};
use serde::{Deserialize, Serialize};

use crate::utils::unix_timestamp::unix_timestamp;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct BotBalance {
    pub accounts: HashMap<String, HashMap<String, Vec<BotBalanceEntry>>>,
    pub timestamp: u64,
}

impl Default for BotBalance {
    fn default() -> Self {
        Self {
            accounts: Default::default(),
            timestamp: unix_timestamp(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct BotBalanceEntry {
    pub coin: String,
    pub amount: Decimal,
}

impl BotBalance {
    pub fn merge_across_exchanges(&self) -> HashMap<String, Vec<BotBalanceEntry>> {
        let mut merged_balances: HashMap<String, HashMap<String, Decimal>> = HashMap::new();

        for (account, exchanges) in &self.accounts {
            for (_exchange, balances) in exchanges {
                for balance in balances {
                    let entry = merged_balances
                        .entry(account.clone())
                        .or_insert_with(HashMap::new)
                        .entry(balance.coin.clone())
                        .or_insert_with(Decimal::zero);

                    *entry += balance.amount;
                }
            }
        }

        let mut result: HashMap<String, Vec<BotBalanceEntry>> = HashMap::new();

        for (account, coins) in merged_balances {
            let entries: Vec<BotBalanceEntry> = coins
                .into_iter()
                .map(|(coin, amount)| BotBalanceEntry { coin, amount })
                .collect();

            result.insert(account, entries);
        }

        result
    }
}
