use std::path::PathBuf;

use clap::Parser;

/// MDH Discord Bot notifier
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Config file path
    #[arg(short, long)]
    pub config_path: PathBuf,
}
