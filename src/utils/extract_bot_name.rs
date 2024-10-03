use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_BOT_EXTRACT: Regex =
        Regex::new(r"hummingbot-([^_]+)-\d{4}\.\d{2}\.\d{2}_\d{2}\.\d{2}").unwrap();
}

pub fn extract_bot_name(bot_name: &str) -> Result<&str> {
    Ok(RE_BOT_EXTRACT
        .captures(bot_name)
        .context("No captures found!")?
        .get(1)
        .context("Regex group not found!")?
        .as_str())
}
