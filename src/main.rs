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
use log::debug;
use log::error;
use log::warn;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ChannelId;
use poise::serenity_prelude::CreateAttachment;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateMessage;
use poise::CreateReply;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use structs::bot_balance::BotBalance;
use structs::extensions::converter::AccountStateConverter;
use structs::extensions::converter::BotsConverter;
use structs::jsonl_cache::JsonCache;
use structs::profit_chart::ChartData;
use structs::profit_chart::ChartDataEntry;
use structs::trade::TradeSide;
use tokio::time::sleep_until;
use tokio::time::Instant;
use tokio_cron_scheduler::{Job, JobScheduler};
use utils::extract_bot_name::extract_bot_name;

struct Data<'c> {
    config: Config<'c>,
    client: Arc<BackendAPIClient>,
    cache: Arc<JsonCache<BotBalance>>,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'c, 'a> = poise::Context<'a, Data<'c>, Error>;

fn make_chart(cache: &JsonCache<BotBalance>) -> Result<Vec<u8>> {
    let mut chart_data = ChartData {
        ..Default::default()
    };
    for balance in cache.get_all_objects().unwrap().iter() {
        let merged_balance = balance.merge_across_exchanges();
        for (account, entries) in merged_balance.iter() {
            if chart_data.chart_data.contains_key(account) {
                let to_add_to = chart_data.chart_data.get_mut(account).unwrap();
                to_add_to.extend(entries.iter().map(|x| ChartDataEntry {
                    timestamp: balance.timestamp,
                    balance: x.amount,
                }));
            } else {
                chart_data.chart_data.insert(
                    account.clone(),
                    entries
                        .iter()
                        .map(|x| ChartDataEntry {
                            timestamp: balance.timestamp,
                            balance: x.amount,
                        })
                        .collect(),
                );
            }
        }
    }
    chart_data.render_chart()
}

/// Test the stats announcement
#[poise::command(
    slash_command,
    prefix_command,
    default_member_permissions = "ADMINISTRATOR"
)]
async fn stats_announcement_test(ctx: Context<'_, '_>) -> Result<(), Error> {
    let builder = CreateReply::default()
        .ephemeral(true)
        .content("Testing stats announcement... Message should arrive soon");
    ctx.send(builder).await?;
    let data = ctx.data();
    let chart_announcement_channel =
        ChannelId::new(data.config.scheduled_chart_announcement.channel_id);

    if data.cache.is_empty() {
        let account_state = data.client.get_account_state().await;
        match account_state {
            Ok(account_state) => {
                let balance_entry = account_state.to_bot_balance();
                data.cache.write(balance_entry).unwrap();
            }
            Err(e) => {
                warn!("Error (Ignored) getting bots from cache: {}", e);
            }
        }
    }

    notify_bot_stats(
        ctx.serenity_context(),
        &data.config.scheduled_chart_announcement.message,
        &data.cache,
        &chart_announcement_channel,
    )
    .await?;
    Ok(())
}

/// Displays a profit chart
#[poise::command(slash_command, prefix_command)]
async fn profit_chart(ctx: Context<'_, '_>) -> Result<(), Error> {
    let reply = ctx.reply("Starting to post the charts!").await?;
    let data = ctx.data();

    if data.cache.is_empty() {
        let account_state = data.client.get_account_state().await;
        match account_state {
            Ok(account_state) => {
                let balance_entry = account_state.to_bot_balance();
                data.cache.write(balance_entry).unwrap();
            }
            Err(e) => {
                warn!("Error (Ignored) getting bots from cache: {}", e);
            }
        }
    }

    let graph = make_chart(&data.cache)?;
    if graph.is_empty() {
        return Ok(());
    }

    ctx.channel_id()
        .send_files(
            ctx,
            vec![CreateAttachment::bytes(graph, "graph.png")],
            CreateMessage::default().content("Current profits / losses"),
        )
        .await?;

    reply
        .edit(ctx, poise::CreateReply::default().content("Done!"))
        .await?;
    Ok(())
}

async fn notify_bot_stats<'c>(
    ctx: &poise::serenity_prelude::Context,
    message: &str,
    cache: &Arc<JsonCache<BotBalance>>,
    channel: &ChannelId,
) -> Result<()> {
    let graph = make_chart(cache)?;
    if graph.is_empty() {
        return Ok(());
    }

    channel
        .send_files(
            ctx,
            vec![CreateAttachment::bytes(graph, "graph.png")],
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
            ("Bot", extract_bot_name(bot_name)?, false),
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
    cache: Arc<JsonCache<BotBalance>>,
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
                let account_state = client.get_account_state().await;
                match account_state {
                    Ok(account_state) => {
                        let balance_entry = account_state.to_bot_balance();
                        cache.write(balance_entry).unwrap();
                        if let Err(e) =
                            notify_bot_stats(&ctx, &message, &cache, &chart_announcement_channel)
                                .await
                        {
                            warn!("Error (Ignored) notifying bot stats: {}", e);
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
    ctx: poise::serenity_prelude::Context,
    config: &Config<'c>,
    client: Arc<BackendAPIClient>,
) -> Result<()> {
    let stats_channel = ChannelId::new(config.stats_channel_id);
    tokio::spawn(async move {
        let mut timestamps: HashMap<String, u64> = HashMap::new();
        loop {
            sleep_until(Instant::now() + Duration::from_secs(10)).await;
            let bots = match client.get_bots().await {
                Ok(response) => response,
                Err(e) => {
                    warn!("get_bots error (ignored): {}", e);
                    continue;
                }
            }
            .to_internal_bots();
            timestamps.retain(|k, _| bots.iter().any(|b| b.name == k.as_str()));
            for bot in bots.into_iter() {
                let latest_trade = bot.get_latest_trade(&client).await;
                match latest_trade {
                    Ok(trade) => {
                        let trade = match trade {
                            Some(trade) => trade,
                            None => {
                                continue;
                            }
                        };
                        let last_timestamp = timestamps.entry(bot.name.to_string()).or_insert(0);
                        if *last_timestamp != trade.timestamp {
                            notify_trade(&ctx, &bot.name, &stats_channel, &trade)
                                .await
                                .unwrap();
                            *last_timestamp = trade.timestamp;
                        }
                    }
                    Err(e) => {
                        error!("Error getting latest trade for bot {}: {}", bot.name, e);
                    }
                }
            }
        }
    });
    Ok(())
}

fn init_config<'c>(path: &PathBuf) -> Result<Config<'c>> {
    if path.exists() {
        let bytes = std::fs::read(path)?;
        let contents = String::from_utf8_lossy(&bytes);
        let config: Config = serde_yaml::from_str(&contents)?;
        fs::create_dir_all(&config.cache_path)?;
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
    let cache = Arc::new(JsonCache::new(config.cache_path.join("balance.jsonl")));

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![profit_chart(), stats_announcement_test()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                trade_loop(ctx.clone(), &config, client.clone()).await?;
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
