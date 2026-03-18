#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::{path::Path, path::PathBuf, str::FromStr, time::Duration};

use clap::{Parser, Subcommand};
use clippers::{Clipboard, ClipperData};
use libconfig::ConfigExt;
use libproduct::product_name;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

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

/// Returns the platform-appropriate directory for log files.
///
/// - macOS:   `~/Library/Logs/clipd/`
/// - Linux:   `$XDG_STATE_HOME/clipd/logs/`  (default: `~/.local/state/clipd/logs/`)
/// - Windows: `%LOCALAPPDATA%\clipd\logs\`
fn log_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join("Library/Logs/clipd")
    }
    #[cfg(target_os = "linux")]
    {
        let state = std::env::var("XDG_STATE_HOME").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{home}/.local/state")
        });
        PathBuf::from(state).join("clipd/logs")
    }
    #[cfg(target_os = "windows")]
    {
        let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
        PathBuf::from(local).join("clipd\\logs")
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        PathBuf::from("/tmp/clipd/logs")
    }
}

/// Initialises the tracing subscriber.
///
/// In daemon mode, logs are written to rolling daily files in [`log_dir()`] with
/// no ANSI colour codes. In interactive mode, logs go to stdout.
///
/// Returns the `WorkerGuard` that must be kept alive for the duration of the
/// program; dropping it flushes and closes the background writer thread.
fn init_tracing(daemon: bool, log_level: Level) -> Option<WorkerGuard> {
    if daemon {
        let dir = log_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("Warning: could not create log directory {}: {e}", dir.display());
        }
        let file_appender = tracing_appender::rolling::daily(&dir, "clipd.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::from_level(log_level)),
            )
            .init();

        Some(guard)
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_filter(tracing_subscriber::filter::LevelFilter::from_level(log_level)),
            )
            .init();

        None
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[allow(clippy::borrow_interior_mutable_const)]
    PRODUCT_NAME.set_global().unwrap();

    let args = Args::parse();

    let log_level = Level::from_str(&std::env::var("RUST_LOG").unwrap_or_default()).unwrap_or(
        if args.verbose { Level::DEBUG } else { Level::INFO },
    );

    // Keep the guard alive for the lifetime of main so the file writer isn't dropped early.
    let _tracing_guard = init_tracing(args.daemon, log_level);

    if let Some(Command::Config) = args.command {
        Config::load().ok();
        let path = libpath::config_path("clipd");
        open_in_editor(&path)?;
        return Ok(());
    }

    let config_path = libpath::config_path("clipd");
    let mut loaded =
        Config::load_tracked().map_err(|e| anyhow::anyhow!("Failed to load config: {e}"))?;

    tracing::info!("clipd started");

    loop {
        // Reload if the config file was modified externally (e.g. via `clipd config`).
        let current_mtime = std::fs::metadata(&config_path)
            .ok()
            .and_then(|m| m.modified().ok());
        if current_mtime != loaded.mtime() {
            match Config::load_tracked() {
                Ok(fresh) => {
                    tracing::info!("Config reloaded");
                    loaded = fresh;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to reload config, keeping previous");
                }
            }
        }

        let result = tick(&loaded).await;

        if !args.daemon {
            result?;
            return Ok(());
        }

        if let Err(e) = result {
            tracing::error!(error = %e, "Tick failed");
        }

        tokio::time::sleep(Duration::from_millis(loaded.tick_interval_ms)).await;
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
    std::process::Command::new("open")
        .args(["-t", path.to_str().unwrap()])
        .status()?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(path).status()?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/c", "start", "", path.to_str().unwrap()])
        .status()?;

    Ok(())
}

async fn tick(config: &Config) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::get();

    let content = match clipboard.read() {
        Some(ClipperData::Text(t)) => t.to_string(),
        _ => return Ok(()),
    };

    // Only log content when it matched a pattern; unmatched clipboard text is never logged.
    let Some(updated_content) = config.apply(&content) else {
        tracing::debug!("No patterns matched");
        return Ok(());
    };

    clipboard
        .write_text(&updated_content)
        .map_err(|e| anyhow::anyhow!("Failed to write clipboard content: {e}"))?;

    tracing::info!(input = %content, output = %updated_content, "Transformed clipboard");

    Ok(())
}
