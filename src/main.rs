mod args;
mod config;
mod structs;
use anyhow::anyhow;
use anyhow::Result;
use args::Args;
use clap::Parser;
use config::Config;
use debounced::debounced;
use futures::SinkExt;
use futures::StreamExt;
use log::debug;
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ChannelId;
use poise::serenity_prelude::CreateAttachment;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateMessage;
use rust_decimal::prelude::*;
use std::fs;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::time::Duration;
use structs::trade::TradeSide;

use crate::structs::extensions::profit_chart_renderer::ProfitChartRenderer;
use crate::structs::extensions::trade_csv_parser::TradeCSVParser;
use crate::structs::profit_chart;
use crate::structs::trade::Trade;

struct Data<'c> {
    config: Config<'c>,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'c, 'a> = poise::Context<'a, Data<'c>, Error>;

/// Displays a profit chart
#[poise::command(slash_command, prefix_command)]
async fn profit_chart(ctx: Context<'_, '_>) -> Result<(), Error> {
    let reply = ctx.reply("Starting to post the charts!").await?;
    let data = ctx.data();
    for (idx, bot) in data.config.bots.iter().enumerate() {
        let file_str = fs::read_to_string(&bot.trades_path)?;
        reply
            .edit(
                ctx,
                poise::CreateReply::default().content(format!(
                    "Posting chart {}/{}",
                    idx + 1,
                    data.config.bots.len()
                )),
            )
            .await?;
        let mut current_buy = Decimal::zero();
        let mut total_profit = Decimal::zero();
        let chart_data = file_str
            .lines()
            .skip(1)
            .map(|line| Trade::from_line(line).unwrap())
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
        drop(file_str);
        let chart = profit_chart::ChartData {
            chart_data,
            base_asset: bot.base_asset.to_owned(),
            bot_name: bot.name.to_owned(),
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
    return Ok(());
}

async fn trade_loop<'c>(ctx: &poise::serenity_prelude::Context, config: &Config<'c>) -> Result<()> {
    for bot in config.bots.iter() {
        let (mut tx, rx) = futures::channel::mpsc::channel(16);
        let mut debounced = debounced(rx, Duration::from_secs(1));
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                futures::executor::block_on(async {
                    tx.send(res).await.unwrap();
                })
            },
            NotifyConfig::default(),
        )?;

        let mut last_file_len = fs::metadata(&bot.trades_path).unwrap().len();
        let trades_path = bot.trades_path.clone();
        let ctx = ctx.clone();
        let stats_channel = ChannelId::new(config.stats_channel_id);
        let bot_name = bot.name.to_string();
        tokio::spawn(async move {
            watcher
                .watch(&trades_path, RecursiveMode::NonRecursive)
                .unwrap();
            loop {
                let event = debounced.next().await;
                if event.is_none() {
                    continue;
                }
                let mut file = fs::File::open(&trades_path).unwrap();
                file.seek(SeekFrom::Start(last_file_len)).unwrap();
                let mut buf = vec![0u8; (file.metadata().unwrap().len() - last_file_len) as usize];
                file.read_exact(&mut buf).unwrap();
                last_file_len = file.metadata().unwrap().len();
                drop(file);
                let line = String::from_utf8_lossy(&buf);
                let trade = Trade::from_line(&line).unwrap();
                notify_trade(&ctx, &bot_name, &stats_channel, trade)
                    .await
                    .unwrap();
            }
        });
        debug!("Set up watcher for {}", bot.trades_path.display());
    }
    return Ok(());
}

fn init_config<'c>(path: &PathBuf) -> Result<Config<'c>> {
    if path.exists() {
        let bytes = std::fs::read(path)?;
        let contents = String::from_utf8_lossy(&bytes);
        let config: Config = serde_yaml::from_str(&contents)?;
        return Ok(config);
    } else {
        let default_config = Config::default();
        let yaml_config = serde_yaml::to_string(&default_config)?;
        fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, &yaml_config)?;
        return Err(anyhow!(
            "No config file found, default file created at {}!",
            path.display()
        ));
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let config = init_config(&args.config_path).unwrap();
    let intents = serenity::GatewayIntents::non_privileged();
    let bot_token = config.bot_token.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![profit_chart()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                trade_loop(ctx, &config).await?;
                Ok(Data { config })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(&bot_token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
