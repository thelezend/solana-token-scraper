//! Handles incoming messages from the Discord event stream.

use std::{path::Path, str::FromStr};

use solana_sdk::pubkey::Pubkey;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{
    filters::Filter,
    photon_util::{self, is_photon_link},
    util::*,
};

/// Errors that can occur when handling a message.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to extract token.
    #[error("Failed to extract token: {0}")]
    ExtractToken(#[from] ExtractTokenError),

    /// Failed to send sniper request.
    #[error("Failed to send sniper request: {0}")]
    FailedToSendSniperRequest(#[from] reqwest::Error),

    /// Detected token file error.
    #[error("Detected token file error: {0}")]
    DetectedTokensFile(#[from] std::io::Error),

    /// Market cap error.
    #[error("Market cap error: {0}")]
    MarketCap(#[from] MarketCapError),
}

/// Handles an incoming Discord message.
///
/// This function processes the message to detect tokens and sends a request to the specified endpoint if a token is found.
///
/// # Arguments
///
/// * `message` - A reference to the `MessageCreate` object.
/// * `detected_tokens_file_path` - A reference to the path of the file where detected tokens are stored.
/// * `discord_filters` - A slice of `DiscordFilter` objects to be checked against.
/// * `rpc_url` - The RPC URL for Solana.
///
/// # Errors
///
/// Returns an `Error` if any step in the process fails.
pub async fn handle_message(
    message: &MessageCreate,
    detected_tokens_file_path: &Path,
    filters: &[Filter],
    rpc_url: &str,
) -> Result<(), Error> {
    if message.guild_id.is_none() {
        return Ok(());
    }

    let filter = match filter_message(message, filters) {
        Some(f) => f,
        None => return Ok(()),
    };

    let token = match process_message_for_token(message, rpc_url).await? {
        Some(t) => t,
        None => return Ok(()),
    };

    if let Some(mc_threshold) = filter.market_cap {
        if !filter_market_cap(&token.to_string(), mc_threshold).await? {
            return Ok(());
        }
    }

    if is_token_already_detected(&token.to_string(), detected_tokens_file_path).await? {
        tracing::debug!("Token {} already detected, skipping", token);
        return Ok(());
    }

    tracing::info!(
        "Token detected for {}: {}",
        filter.name,
        console::style(token).green()
    );

    send_token_request(&token.to_string(), &filter.token_endpoint_url).await?;
    add_token_to_file(&token.to_string(), detected_tokens_file_path).await?;
    tracing::debug!("Successfully sent request to endpoint for token: {}", token);

    Ok(())
}

/// Filters a message based on the provided filters.
///
/// This function iterates through the provided `filters` and checks if the message matches any of the filters.
/// If a match is found, the corresponding `Filter` is returned.
///
/// # Arguments
///
/// * `message` - A reference to the `MessageCreate` object.
/// * `filters` - A slice of `Filter` objects to be checked against.
///
/// # Returns
///
/// * `Some(Filter)` if a matching filter is found, otherwise `None`.
fn filter_message(message: &MessageCreate, filters: &[Filter]) -> Option<Filter> {
    for filter in filters {
        match (filter.discord_channel_id, filter.discord_user_id) {
            (Some(channel_id), Some(user_id)) => {
                if message.channel_id.get() == channel_id && message.author.id.get() == user_id {
                    return Some(filter.clone());
                }
            }
            (Some(channel_id), None) => {
                if message.channel_id.get() == channel_id {
                    return Some(filter.clone());
                }
            }
            (None, Some(user_id)) => {
                if message.author.id.get() == user_id {
                    return Some(filter.clone());
                }
            }
            _ => {
                return Some(filter.clone());
            }
        }
    }
    None
}

/// Processes a message to extract a token.
///
/// This function attempts to extract a token from the message content or the descriptions of the message embeds.
///
/// # Arguments
///
/// * `message` - A reference to the `MessageCreate` object.
///
/// # Errors
///
/// Returns an `ExtractTokenError` if the token extraction fails.
async fn process_message_for_token(
    message: &MessageCreate,
    rpc_url: &str,
) -> Result<Option<Pubkey>, ExtractTokenError> {
    // Attempt to extract a token from the message content
    if let Some(token) = extract_token(&message.content, rpc_url).await? {
        return Ok(Some(token));
    }

    // Attempt to extract a token from the descriptions of the message embeds
    for embed in &message.embeds {
        if let Some(description) = &embed.description {
            if let Some(token) = extract_token(description, rpc_url).await? {
                return Ok(Some(token));
            }
        }
    }

    Ok(None)
}

/// Errors that can occur when extracting a token.
///
/// This enum represents the possible errors that can occur during the token extraction process.
#[derive(Debug, thiserror::Error)]
pub enum ExtractTokenError {
    /// Error from the `extract_token_from_pumpfun_link` function.
    #[error("Failed to extract token from pumpfun link: {0}")]
    ExtractTokenFromPumpFunLink(#[from] super::util::ExtractTokenFromPumpFunLinkError),

    /// Error from the `photon_util::fetch_token` function.
    #[error("Failed to fetch token from photon link: {0}")]
    PhotonFetchToken(#[from] super::photon_util::FetchTokenError),
}

/// Extracts a token from the given content.
///
/// This function checks each word in the content to see if it is a valid token address,
/// a Pump.fun link, or a Photon link, and extracts the token if found.
///
/// # Arguments
///
/// * `content` - A string slice that holds the content to be checked.
///
/// # Errors
///
/// Returns an `ExtractTokenError` if the token extraction fails.
pub async fn extract_token(
    content: &str,
    rpc_url: &str,
) -> Result<Option<Pubkey>, ExtractTokenError> {
    for word in content.split_whitespace() {
        if is_valid_token_address(word) {
            return Ok(Some(Pubkey::from_str(word).unwrap()));
        }
        if is_pumpfun_link(word) {
            return Ok(Some(extract_token_from_pumpfun_link(word)?));
        }
        if is_photon_link(word) {
            return Ok(Some(photon_util::fetch_token(word, rpc_url).await?));
        }
    }

    Ok(None)
}
