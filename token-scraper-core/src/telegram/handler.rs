//! Telegram handler module.

use std::path::Path;

use grammers_client::Update;

use crate::{
    filters::Filter,
    message_handler::extract_token,
    util::{add_token_to_file, filter_market_cap, is_token_already_detected, send_token_request},
};

use super::{errors::*, util::*};

/// Starts the Telegram client and processes incoming messages.
///
/// This function connects to the Telegram client, authorizes it, and continuously listens for new messages.
/// If a message matches any of the provided filters, it extracts the token and sends a request to the specified endpoint.
///
/// # Arguments
///
/// * `api_id` - The API ID for the Telegram client.
/// * `api_hash` - The API hash for the Telegram client.
/// * `filters` - A slice of `Filter` objects to be checked against.
/// * `solana_rpc_url` - The RPC URL for Solana.
/// * `detected_tokens_file_path` - A reference to the path of the file where detected tokens are stored.
///
/// # Errors
///
/// Returns a `TelegramError` if any step in the process fails.
pub async fn start(
    client: &grammers_client::Client,
    filters: &[Filter],
    solana_rpc_url: &str,
    detected_tokens_file_path: &Path,
) -> Result<(), TelegramError> {
    tracing::info!("Connected to Telegram!");

    loop {
        let update = client.next_update().await?;
        match update {
            Update::NewMessage(message) if !message.outgoing() => {
                let filters = filters.to_vec();
                let solana_rpc_url = solana_rpc_url.to_string();
                let detected_tokens_file_path = detected_tokens_file_path.to_path_buf();

                tokio::spawn(async move {
                    let result = process_message(
                        message,
                        &filters,
                        &solana_rpc_url,
                        &detected_tokens_file_path,
                    )
                    .await;

                    if let Err(e) = result {
                        tracing::error!("Failed to process message: {:?}", e);
                    }
                });
            }
            _ => {}
        }
    }
}

/// Processes a Telegram message to detect and handle tokens.
///
/// This function checks if the message matches any of the provided filters. If a match is found,
/// it extracts the token from the message, sends a request to the token endpoint, and adds the token
/// to the specified file.
///
/// # Arguments
///
/// * `message` - The Telegram message to be processed.
/// * `filters` - A slice of `Filter` objects to be checked against.
/// * `solana_rpc_url` - The RPC URL for Solana.
/// * `detected_tokens_file_path` - A reference to the path of the file where detected tokens are stored.
///
/// # Errors
///
/// Returns a `TelegramError` if any step in the process fails.
async fn process_message(
    message: grammers_client::types::Message,
    filters: &[Filter],
    solana_rpc_url: &str,
    detected_tokens_file_path: &Path,
) -> Result<(), TelegramError> {
    let channel_id = message.chat().id();
    if let Some(filter) = filter_message(channel_id, filters) {
        let token = extract_token(message.text(), solana_rpc_url).await?;
        if let Some(token) = token {
            tracing::info!(
                "Token detected for {}: {}",
                filter.name,
                console::style(token).green()
            );

            if let Some(market_cap) = filter.market_cap {
                if !filter_market_cap(&token.to_string(), market_cap).await? {
                    return Ok(());
                }
            }

            if is_token_already_detected(&token.to_string(), detected_tokens_file_path).await? {
                tracing::info!("Token {} already detected, skipping", token);
                return Ok(());
            }

            send_token_request(&token.to_string(), &filter.token_endpoint_url).await?;
            add_token_to_file(&token.to_string(), detected_tokens_file_path).await?;
            tracing::debug!("Successfully sent request to endpoint for token: {}", token);
        } else {
            tracing::debug!("No token detected in message for {}", filter.name);
        }
    }

    Ok(())
}
