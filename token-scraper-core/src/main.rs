//! Main module for the solana-token-scraper program.
//!
//! This module sets up logging, starts the event stream, and handles incoming events.

#![warn(
    missing_docs,
    rustdoc::unescaped_backticks,
    clippy::missing_errors_doc,
    clippy::missing_docs_in_private_items
)]

mod discord;
mod filters;
mod macros;
mod message_handler;
mod photon_util;
mod settings;
mod telegram;
mod util;

use std::{path::Path, sync::Arc};

use discord::stream::start_stream;
use filters::read_filters_from_csv;
use message_handler::handle_message;
use tokio::sync::{mpsc, Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use twilight_model::gateway::event::DispatchEvent;

use crate::settings::Settings;

/// Path to the detected tokens file.
pub const DETECTED_TOKENS_FILE_PATH: &str = "detected_tokens.txt";

/// Path to the Discord filters file.
pub const DISCORD_FILTERS_FILE_PATH: &str = "discord_filters.csv";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::new()?;
    let filters = read_filters_from_csv(Path::new(DISCORD_FILTERS_FILE_PATH))?;
    // Create the detected tokens file if it doesn't exist
    if !Path::new(DETECTED_TOKENS_FILE_PATH).exists() {
        std::fs::File::create(DETECTED_TOKENS_FILE_PATH)?;
    }

    // Setup logging
    // Create a file layer with info level filtering
    let file_appender = tracing_appender::rolling::daily("logs", "token-scraper.log");
    let (file_writer, _file_writer_guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(file_writer)
        .with_filter(EnvFilter::new("info"));
    // .with_filter(EnvFilter::new("debug"));

    tracing_subscriber::registry().with(file_layer).init();

    let (discord_event_tx, mut discord_event_rx) = mpsc::unbounded_channel();

    // Spawn a task to manage the Discord event stream
    tokio::spawn(manage_discord_stream(
        settings.discord.user_token.clone(),
        settings.discord.sec_ws_key.clone(),
        discord_event_tx,
    ));

    // Main event loop
    while let Some(event) = discord_event_rx.recv().await {
        let filters = filters.clone();
        let rpc_url = settings.solana.rpc_url.clone();

        tokio::spawn(async move {
            if let DispatchEvent::MessageCreate(message) = event {
                if let Err(e) = handle_message(
                    &message,
                    Path::new(DETECTED_TOKENS_FILE_PATH),
                    &filters,
                    &rpc_url,
                )
                .await
                {
                    tracing::error!("Error while handling discord message: {:?}", e);
                }
            }
        });
    }

    Ok(())
}

/// Manages the Discord event stream, attempting to reconnect on failures.
async fn manage_discord_stream(
    discord_token: String,
    sec_ws_key: String,
    event_tx: mpsc::UnboundedSender<DispatchEvent>,
) {
    let attempts = Arc::new(Mutex::new(0));
    loop {
        tracing::info!("Attempting to start stream");

        // Reset attempts every 60 seconds
        let attempts_clone = Arc::clone(&attempts);
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            *attempts_clone.lock().await = 0;
        });

        let result = start_stream(&discord_token, &sec_ws_key, Arc::new(event_tx.clone())).await;
        if let Err(e) = result {
            tracing::debug!("Stream error: {e:?}");
            println!("Connection closed, retrying...");
            let mut attempts = attempts.lock().await;
            *attempts += 1;
            if *attempts >= 3 {
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    tracing::error!("Stream errored after 3 attempts");
    println!(
        "{}",
        console::style(
            "Discord stream errored after multiple retries. Please check logs for more details and try restarting the program."
        )
        .red()
    );
}
