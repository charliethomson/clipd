use std::{str::FromStr, time::Duration};

use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use libconfig::ConfigExt;
use libproduct::product_name;
use tracing::Level;

use crate::config::Config;

mod config;
mod strategies;

product_name!("dev.thmsn.clipd");

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    daemon: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[allow(clippy::borrow_interior_mutable_const)]
    PRODUCT_NAME.set_global().unwrap();

    let args = Args::parse();

    let default_log_level = if args.verbose {
        Level::DEBUG
    } else {
        Level::ERROR
    };

    let log_level = Level::from_str(&std::env::var("RUST_LOG").unwrap_or_default())
        .unwrap_or(default_log_level);

    tracing_subscriber::fmt().with_max_level(log_level).init();

    let config = Config::default();

    let mut ctx: ClipboardContext = match ClipboardProvider::new() {
        Ok(ctx) => ctx,
        Err(e) => anyhow::bail!("Failed to acquire clipboard handle: {e}"),
    };

    loop {
        let result = tick(&mut ctx).await;

        if !args.daemon {
            result?;
            return Ok(());
        }

        if let Err(e) = result
            && args.verbose
        {
            eprintln!("Failed to tick: {e}")
        }

        tokio::time::sleep(Duration::from_millis(config.tick_interval_ms)).await;
    }
}

async fn tick(ctx: &mut ClipboardContext) -> anyhow::Result<()> {
    let config = Config::load().map_err(|e| anyhow::anyhow!("Failed to load config: {e}"))?;
    let content = ctx
        .get_contents()
        .map_err(|e| anyhow::anyhow!("Failed to read clipboard content: {e}"))?;

    let Some(updated_content) = config.apply(&content) else {
        return Ok(());
    };

    ctx.set_contents(updated_content.clone())
        .map_err(|e| anyhow::anyhow!("Failed to write clipboard content: {e}"))?;

    println!("{content} => {updated_content}");
    Ok(())
}
