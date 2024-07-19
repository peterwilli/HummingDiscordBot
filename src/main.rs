mod args;
mod backend_api;
mod config;
mod structs;
#[cfg(test)]
mod tests;

use anyhow::anyhow;
use anyhow::Result;
use args::Args;
use backend_api::client::BackendAPIClient;
use clap::Parser;
use config::Config;


use futures::StreamExt;

use log::error;
use notify::{Watcher};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ChannelId;
use poise::serenity_prelude::CreateAttachment;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateMessage;
use rust_decimal::prelude::*;
use std::fs;
use std::io::Read;


use std::path::PathBuf;
use std::sync::Arc;

use structs::extensions::converter::BotsConverter;
use structs::trade::TradeSide;

use crate::structs::extensions::profit_chart_renderer::ProfitChartRenderer;
use crate::structs::profit_chart;
use crate::structs::trade::Trade;

struct Data<'c> {
    config: Config<'c>,
    client: Arc<BackendAPIClient>,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'c, 'a> = poise::Context<'a, Data<'c>, Error>;

/// Displays a profit chart
#[poise::command(slash_command, prefix_command)]
async fn profit_chart(ctx: Context<'_, '_>) -> Result<(), Error> {
    let reply = ctx.reply("Starting to post the charts!").await?;
    let data = ctx.data();
    let bots = data.client.get_bots().await?;
    let bots = bots.to_internal_bots()?;
    for (idx, bot) in bots.iter().enumerate() {
        reply
            .edit(
                ctx,
                poise::CreateReply::default().content(format!(
                    "Posting chart {}/{}",
                    idx + 1,
                    bots.len()
                )),
            )
            .await?;
        let mut current_buy = Decimal::zero();
        let mut total_profit = Decimal::zero();
        let trades = bot.get_trades(&data.client).await?;
        let first_trade = match trades.first() {
            Some(trade) => trade,
            None => continue,
        };
        let chart_data = trades
            .iter()
            .filter_map(|trade| match trade.side {
                TradeSide::Buy => {
                    current_buy = trade.amount * trade.price;
                    None
                }
                TradeSide::Sell => {
                    if current_buy == Decimal::zero() {
                        current_buy = trade.amount * trade.price;
                    }
                    total_profit += (trade.amount * trade.price) - current_buy;
                    Some(profit_chart::ChartDataEntry {
                        timestamp: trade.timestamp,
                        profit: total_profit,
                    })
                }
            })
            .collect::<Vec<profit_chart::ChartDataEntry>>();
        let chart = profit_chart::ChartData {
            chart_data,
            base_asset: first_trade.quote_asset.clone(),
            bot_name: bot.name.clone(),
        };
        let png_data = chart.render_chart()?;
        ctx.channel_id()
            .send_files(
                ctx,
                vec![CreateAttachment::bytes(
                    png_data,
                    format!("{}.png", bot.name),
                )],
                CreateMessage::default().content(&format!("Profit chart for **{}**", bot.name)),
            )
            .await?;
    }
    reply
        .edit(ctx, poise::CreateReply::default().content("Done!"))
        .await?;
    Ok(())
}

async fn notify_trade<'c>(
    ctx: &poise::serenity_prelude::Context,
    bot_name: &str,
    channel: &ChannelId,
    trade: Trade<'c>,
) -> Result<()> {
    let embed = CreateEmbed::new()
        .title("New trade")
        .color(match trade.side {
            TradeSide::Buy => 0x41d321,
            TradeSide::Sell => 0xd32121,
        })
        .description(format!(
            "{} {}/{}",
            trade.side, trade.base_asset, trade.quote_asset
        ))
        .fields(vec![
            ("Bot", bot_name, false),
            ("Amount", &trade.amount.to_string(), true),
            (
                "Price",
                format!("{} {}", trade.price, trade.quote_asset).as_ref(),
                true,
            ),
        ]);
    let builder = CreateMessage::new().add_embed(embed);
    channel.send_message(ctx, builder).await?;
    Ok(())
}

async fn trade_loop<'c>(
    ctx: &poise::serenity_prelude::Context,
    config: &Config<'c>,
    client: Arc<BackendAPIClient>,
) -> Result<()> {
    let bots = client.get_bots().await?.to_internal_bots()?;
    for bot in bots.into_iter() {
        let ctx = ctx.clone();
        let stats_channel = ChannelId::new(config.stats_channel_id);
        let bot_name = bot.name.to_string();
        let client = client.clone();
        tokio::spawn(async move {
            loop {
                let latest_trade = bot.get_latest_trade(&client).await;
                match latest_trade {
                    Ok(trade) => {
                        let trade = match trade {
                            Some(trade) => trade,
                            None => {
                                continue;
                            }
                        };

                        notify_trade(&ctx, &bot_name, &stats_channel, trade)
                            .await
                            .unwrap();
                    }
                    Err(e) => {
                        error!("Error getting latest trade for bot {}: {}", bot_name, e);
                    }
                }
            }
        });
    }
    Ok(())
}

fn init_config<'c>(path: &PathBuf) -> Result<Config<'c>> {
    if path.exists() {
        let bytes = std::fs::read(path)?;
        let contents = String::from_utf8_lossy(&bytes);
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    } else {
        let default_config = Config::default();
        let yaml_config = serde_yaml::to_string(&default_config)?;
        fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, yaml_config)?;
        Err(anyhow!(
            "No config file found, default file created at {}!",
            path.display()
        ))
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let config = init_config(&args.config_path).unwrap();
    let intents = serenity::GatewayIntents::non_privileged();
    let bot_token = config.bot_token.clone();
    let client = Arc::new(BackendAPIClient::new(config.backend_api_base_url.clone()));

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![profit_chart()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                trade_loop(ctx, &config, client.clone()).await?;
                Ok(Data { config, client })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(&bot_token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
