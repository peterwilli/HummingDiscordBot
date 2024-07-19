mod args;
mod backend_api;
mod config;
mod structs;
#[cfg(test)]
mod tests;
mod utils;

use crate::structs::extensions::profit_chart_renderer::ProfitChartRenderer;
use crate::structs::trade::Trade;
use anyhow::anyhow;
use anyhow::Result;
use args::Args;
use backend_api::client::BackendAPIClient;
use clap::Parser;
use config::Config;
use futures::StreamExt;

use log::debug;
use log::error;
use log::warn;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ChannelId;
use poise::serenity_prelude::CreateAttachment;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateMessage;

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use structs::bot::Bot;

use structs::bot_cache::BotCache;
use structs::bot_cache::ControllerPNLHistoryEntry;
use structs::extensions::converter::BotsConverter;
use structs::profit_chart::ChartData;
use structs::profit_chart::ChartDataEntry;
use structs::trade::TradeSide;
use tokio::time::sleep_until;
use tokio::time::Instant;
use tokio_cron_scheduler::{Job, JobScheduler};
use utils::beautify_bot_name::beautify_bot_name;
use utils::unix_timestamp::unix_timestamp;

struct Data<'c> {
    config: Config<'c>,
    client: Arc<BackendAPIClient>,
    cache: Arc<BotCache>,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'c, 'a> = poise::Context<'a, Data<'c>, Error>;

fn make_chart(bot: &Bot<'_>, cache: &BotCache) -> Result<Vec<u8>> {
    let mut chart_data = ChartData {
        ..Default::default()
    };
    let cache_entry = cache.get_entry(&bot.name);
    if cache_entry.controllers.is_empty() {
        return Err(anyhow!("No controllers found for bot {}", bot.name));
    }
    for (k, controller) in cache_entry.controllers.iter() {
        chart_data.chart_data.insert(
            k.to_owned(),
            controller
                .pnl_history
                .iter()
                .map(|h| ChartDataEntry {
                    timestamp: h.timestamp,
                    profit: h.pct,
                })
                .collect(),
        );
    }
    chart_data.render_chart()
}

/// Displays a profit chart
#[poise::command(slash_command, prefix_command)]
async fn profit_chart(ctx: Context<'_, '_>) -> Result<(), Error> {
    let reply = ctx.reply("Starting to post the charts!").await?;
    let data = ctx.data();
    let bots = data.client.get_bots().await?;
    let bots = bots.to_internal_bots();
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
        let png_data = make_chart(bot, &data.cache)?;
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

async fn notify_bot_stats<'c>(
    ctx: &poise::serenity_prelude::Context,
    message: &str,
    cache: &BotCache,
    client: &BackendAPIClient,
    channel: &ChannelId,
) -> Result<()> {
    let bots = client.get_bots().await?.to_internal_bots();
    let graphs = bots
        .iter()
        .filter_map(|b| {
            let chart = make_chart(b, cache);
            match chart {
                Ok(chart) => {
                    Some((b.name.to_string(), chart))
                }
                Err(e) => {
                    warn!("Chart error (ignored): {}", e);
                    None
                }
            }
        })
        .collect::<HashMap<String, Vec<u8>>>();

    if graphs.is_empty() {
        return Ok(());
    }

    channel
        .send_files(
            ctx,
            graphs
                .into_iter()
                .map(|(name, bytes)| CreateAttachment::bytes(bytes, format!("{}.png", name)))
                .collect::<Vec<CreateAttachment>>(),
            CreateMessage::default().content(message),
        )
        .await?;

    Ok(())
}

async fn notify_trade<'c>(
    ctx: &poise::serenity_prelude::Context,
    bot_name: &str,
    channel: &ChannelId,
    trade: &Trade<'c>,
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
            ("Bot", beautify_bot_name(bot_name).as_str(), false),
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

async fn pnl_cache_loop<'c>(
    ctx: poise::serenity_prelude::Context,
    config: &Config<'c>,
    client: Arc<BackendAPIClient>,
    cache: Arc<BotCache>,
) -> Result<()> {
    let sched = JobScheduler::new().await?;
    let message = config
        .scheduled_chart_announcement
        .message
        .to_string()
        .clone();
    let chart_announcement_channel = ChannelId::new(config.scheduled_chart_announcement.channel_id);
    let schedule = config.scheduled_chart_announcement.schedule.to_string();
    sched
        .add(Job::new_async(schedule.as_str(), move |uuid, mut l| {
            let client = client.clone();
            let cache = cache.clone();
            let message = message.clone();
            let ctx = ctx.clone();
            Box::pin(async move {
                let bots = client.get_bots().await;
                match bots {
                    Ok(bots) => {
                        let bots = bots.to_internal_bots();
                        for bot in bots.iter() {
                            let mut cache_entry = cache.get_entry(&bot.name);
                            for (name, controller) in bot.controllers.iter() {
                                let controller_entry =
                                    cache_entry.controllers.entry(name.to_owned()).or_default();
                                controller_entry
                                    .pnl_history
                                    .push(ControllerPNLHistoryEntry {
                                        timestamp: unix_timestamp(),
                                        pct: controller.pnl.pct,
                                        quote: controller.pnl.quote,
                                    })
                            }
                            cache.save_entry(&bot.name, cache_entry);
                        }
                        match notify_bot_stats(
                            &ctx,
                            &message,
                            &cache,
                            &client,
                            &chart_announcement_channel,
                        )
                        .await
                        {
                            Err(e) => {
                                warn!("Error (Ignored) notifying bot stats: {}", e);
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        warn!("Error (Ignored) getting bots from cache: {}", e);
                    }
                }

                // Query the next execution time for this job
                let next_tick = l.next_tick_for_job(uuid).await;
                match next_tick {
                    Ok(Some(ts)) => debug!("Next time for job is {}", ts),
                    _ => debug!("Could not get next tick for job"),
                }
            })
        })?)
        .await?;
    sched.start().await?;
    Ok(())
}

async fn trade_loop<'c>(
    ctx: &poise::serenity_prelude::Context,
    config: &Config<'c>,
    client: Arc<BackendAPIClient>,
) -> Result<()> {
    let bots = client.get_bots().await?.to_internal_bots();
    for bot in bots.into_iter() {
        let ctx = ctx.clone();
        let stats_channel = ChannelId::new(config.stats_channel_id);
        let bot_name = bot.name.to_string();
        let client = client.clone();
        tokio::spawn(async move {
            let mut last_timestamp: u64 = 0;
            loop {
                sleep_until(Instant::now() + Duration::from_secs(1)).await;
                let latest_trade = bot.get_latest_trade(&client).await;
                match latest_trade {
                    Ok(trade) => {
                        let trade = match trade {
                            Some(trade) => trade,
                            None => {
                                continue;
                            }
                        };
                        if last_timestamp != trade.timestamp {
                            notify_trade(&ctx, &bot_name, &stats_channel, &trade)
                                .await
                                .unwrap();
                            last_timestamp = trade.timestamp;
                        }
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
    let cache = Arc::new(BotCache::new(config.cache_path.clone()));

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![profit_chart()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                trade_loop(ctx, &config, client.clone()).await?;
                if config.scheduled_chart_announcement.enabled {
                    pnl_cache_loop(ctx.clone(), &config, client.clone(), cache.clone()).await?;
                }
                Ok(Data {
                    config,
                    client,
                    cache,
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(&bot_token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
