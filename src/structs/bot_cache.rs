use std::{collections::HashMap, path::PathBuf};

use log::debug;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::utils::extract_bot_name::extract_bot_name;

#[derive(Serialize, Deserialize, Default)]
pub struct ControllerPNLHistoryEntry {
    pub timestamp: u64,
    pub pct: Decimal,
    pub quote: Decimal,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Controller {
    pub pnl_history: Vec<ControllerPNLHistoryEntry>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct BotCacheEntry {
    pub controllers: HashMap<String, Controller>,
}

pub struct BotCache {
    cache_path: PathBuf,
    strip_bot_name: bool,
}

impl BotCache {
    pub fn new(cache_path: PathBuf, strip_bot_name: bool) -> Self {
        // check if cache exists, if not, make the folder
        if !cache_path.exists() {
            std::fs::create_dir_all(&cache_path).unwrap();
            debug!("Created cache folder {}", cache_path.display());
        }
        Self {
            cache_path,
            strip_bot_name,
        }
    }

    pub fn get_entry(&self, bot_name: &str) -> BotCacheEntry {
        let bot_name = if self.strip_bot_name {
            extract_bot_name(bot_name).unwrap()
        } else {
            bot_name
        };
        let file_path = self.cache_path.join(format!("{}.json", bot_name));
        if !file_path.exists() {
            return BotCacheEntry::default();
        }
        let file_contents = std::fs::read_to_string(&file_path).unwrap();
        let bot_cache: BotCacheEntry = serde_json::from_str(&file_contents).unwrap();
        bot_cache
    }

    pub fn save_entry(&self, bot_name: &str, bot_cache: BotCacheEntry) {
        let bot_name = if self.strip_bot_name {
            extract_bot_name(bot_name).unwrap()
        } else {
            bot_name
        };
        let file_path = self.cache_path.join(format!("{}.json", bot_name));
        let file_contents = serde_json::to_string(&bot_cache).unwrap();
        std::fs::write(&file_path, file_contents).unwrap();
        debug!("Saved bot cache to {}", file_path.to_str().unwrap());
    }
}
