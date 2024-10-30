//! Contains the logic to process WebSocket messages and detect tokens.

use super::{errors::WsClientError, types::MonitorData};
use crate::{
    message_handler::extract_token,
    util::{add_token_to_file, is_token_already_detected, send_token_request},
};
use std::path::Path;

/// Processes a WebSocket message to detect and handle tokens.
///
/// # Arguments
///
/// * `message` - The WebSocket message containing monitor data.
/// * `solana_rpc_url` - The URL of the Solana RPC endpoint.
/// * `detected_tokens_file_path` - The file path where detected tokens are stored.
/// * `token_endpoint_url` - The URL of the endpoint to send detected tokens.
///
/// # Returns
///
/// * `Result<(), WsClientError>` - Returns an empty result on success or a `WsClientError` on failure.
pub async fn process_message(
    message: &MonitorData,
    solana_rpc_url: &str,
    detected_tokens_file_path: &Path,
    token_endpoint_url: &str,
) -> Result<(), WsClientError> {
    // Extract token from the message data
    let token = extract_token(message.data.as_str().unwrap(), solana_rpc_url).await?;

    if let Some(token) = token {
        tracing::info!(
            "Token detected on ws from user {}: {}",
            message.task.user,
            console::style(token).green()
        );

        // Check if the token is already detected
        if is_token_already_detected(&token.to_string(), detected_tokens_file_path).await? {
            tracing::info!("Token {} already detected, skipping", token);
            return Ok(());
        }

        // Send the token to the specified endpoint
        send_token_request(&token.to_string(), token_endpoint_url).await?;

        // Add the token to the detected tokens file
        add_token_to_file(&token.to_string(), detected_tokens_file_path).await?;

        tracing::debug!("Successfully sent request to endpoint for token: {}", token);
    } else {
        tracing::debug!("No token detected in ws message {:?}", message);
    }

    Ok(())
}
