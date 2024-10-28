//! Discord stream module for the token-scraper application.
//!
//! This module handles the connection to the Discord API and the WebSocket stream.

mod handler;
mod identify;
mod util;
mod ws_request;

use std::sync::Arc;

use futures_util::StreamExt;
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use tokio_tungstenite::connect_async;
use twilight_model::gateway::event::DispatchEvent;

use self::handler::handle_stream;

/// Error types for the Discord stream module.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error creating the initial websocket request.
    #[error("Failed to create initial websocket request: {0}")]
    InitalWsRequest(#[from] ws_request::Error),

    /// Error initializing the websocket.
    #[error("Failed to initialize websocket: {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// Error during identification.
    #[error("Failed to identify: {0}")]
    Identify(#[from] identify::Error),

    /// Error handling the stream.
    #[error("Failed to handle stream: {0}")]
    Stream(#[from] handler::Error),
}

/// Starts the Discord WebSocket stream.
///
/// This function creates a WebSocket request with the provided security key,
/// connects to the Discord gateway, and handles the stream.
///
/// # Errors
///
/// This function will return an error if the WebSocket request creation fails,
/// the WebSocket connection fails, or if handling the stream fails.
pub async fn start_stream(
    discord_token: &str,
    sec_ws_key: &str,
    event_tx: Arc<UnboundedSender<DispatchEvent>>,
) -> Result<(), Error> {
    let request = ws_request::create_request_with_headers(sec_ws_key.to_string()).await?;
    let (ws_stream, _) = connect_async(request).await?;
    let (ws_write, ws_read) = ws_stream.split();

    let ws_write = Arc::new(Mutex::new(ws_write));

    let sequence: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(None));
    let resume_gateway_url: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    handle_stream(
        discord_token,
        ws_read,
        ws_write,
        event_tx,
        sequence,
        resume_gateway_url,
        session_id,
    )
    .await?;

    Ok(())
}
