use std::{path::Path, str::FromStr, time::Duration};

use clap::{Parser, Subcommand};
use clipboard::{ClipboardContext, ClipboardProvider};
use libconfig::ConfigExt;
use libproduct::product_name;
use tracing::Level;

use crate::config::Config;

mod config;
mod strategies;

product_name!("dev.thmsn.clipd");

/// clipd — watches your clipboard and transforms its contents using configurable rules.
///
/// Run without arguments to process the clipboard once and exit.
/// Pass --daemon to run continuously in the background.
///
/// Configuration is stored at the platform config path for dev.thmsn.clipd.
/// Run `clipd config` to open it in your default editor.
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Enable debug logging.
    #[arg(short, long)]
    verbose: bool,

    /// Run continuously, polling the clipboard on the configured interval.
    #[arg(short, long)]
    daemon: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Open the clipd configuration file in your default editor.
    ///
    /// Creates the file with defaults if it does not yet exist.
    /// Respects the EDITOR and VISUAL environment variables; falls back to
    /// the platform default (open on macOS, xdg-open on Linux, start on Windows).
    Config,
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

    if let Some(Command::Config) = args.command {
        // Ensure the config file exists before opening it.
        Config::load().ok();
        let path = libpath::config_path("clipd");
        open_in_editor(&path)?;
        return Ok(());
    }

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

fn open_in_editor(path: &Path) -> anyhow::Result<()> {
    let editor = std::env::var("EDITOR")
        .ok()
        .or_else(|| std::env::var("VISUAL").ok());

    if let Some(editor) = editor {
        std::process::Command::new(&editor).arg(path).status()?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").args(["-t", path.to_str().unwrap()]).status()?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(path).status()?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/c", "start", "", path.to_str().unwrap()])
        .status()?;

    Ok(())
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
