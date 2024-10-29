//! Identify module for the Discord stream.
//!
//! This module handles the identification process for the Discord WebSocket stream.

use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite;

use super::handler::WebsocketWrite;

/// Error types for the identify module.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// WebSocket error.
    #[error("WebSocket error: {0}")]
    WsError(#[from] tokio_tungstenite::tungstenite::Error),
}

/// Identifies the client with the Discord WebSocket.
///
/// This function sends an identity payload to the Discord WebSocket to identify the client.
///
/// # Errors
///
/// This function will return an error if the WebSocket send operation fails.
pub async fn identify(discord_token: String, ws_write: WebsocketWrite) -> Result<(), Error> {
    let identity_payload = Identity::new(&discord_token);
    let mut ws_write = ws_write.lock().await;

    ws_write
        .send(tungstenite::Message::Text(
            serde_json::to_string(&identity_payload).unwrap(),
        ))
        .await?;

    tracing::debug!("Identified with Discord");

    Ok(())
}

/// Represents the identity payload sent to the Discord WebSocket.
#[derive(Serialize, Deserialize)]
pub struct Identity {
    /// Operation code for the payload.
    op: u8,
    /// Data field containing the identity information.
    d: Data,
}

/// Contains the data required for identifying the client.
#[derive(Serialize, Deserialize)]
struct Data {
    /// Discord token for authentication.
    token: String,
    /// Capabilities of the client.
    capabilities: u32,
    /// Properties of the client.
    properties: Properties,
    /// Presence information of the client.
    presence: Presence,
    /// Whether to compress the data.
    compress: bool,
    /// State of the client.
    client_state: ClientState,
}

/// Properties of the client.
///
/// This struct contains various properties of the client, such as the operating system,
/// browser, device, and other relevant information.
#[derive(Serialize, Deserialize)]
struct Properties {
    /// Operating system of the client.
    os: String,
    /// Browser used by the client.
    browser: String,
    /// Device used by the client.
    device: String,
    /// System locale of the client.
    system_locale: String,
    /// User agent of the browser.
    browser_user_agent: String,
    /// Version of the browser.
    browser_version: String,
    /// Version of the operating system.
    os_version: String,
    /// Referrer URL.
    referrer: String,
    /// Referring domain.
    referring_domain: String,
    /// Current referrer URL.
    referrer_current: String,
    /// Current referring domain.
    referring_domain_current: String,
    /// Release channel of the client.
    release_channel: String,
    /// Build number of the client.
    client_build_number: u32,
    /// Source of the client event, if any.
    client_event_source: Option<String>,
}

/// Represents the presence information of the client.
#[derive(Serialize, Deserialize)]
struct Presence {
    /// Status of the client (e.g., online, offline).
    status: String,
    /// Timestamp of the last status update.
    since: u64,
    /// List of activities the client is engaged in.
    activities: Vec<Activity>,
    /// Whether the client is away from keyboard.
    afk: bool,
}

/// Represents an activity the client is engaged in.
#[derive(Serialize, Deserialize)]
struct Activity {
    // Define fields if needed
}

/// Represents the state of the client.
#[derive(Serialize, Deserialize)]
struct ClientState {
    /// Versions of the guilds the client is in.
    guild_versions: std::collections::HashMap<String, u32>,
}

impl Identity {
    /// Creates a new `Identity` instance with the provided token.
    ///
    /// This function initializes the `Identity` struct with default values for
    /// properties, presence, and client state.
    ///
    /// # Errors
    ///
    /// This function does not return any errors.
    pub fn new(token: &str) -> Self {
        Identity {
            op: 2,
            d: Data {
                token: token.to_string(),
                capabilities: 30717,
                properties: Properties {
                    os: "Mac OS X".to_string(),
                    browser: "Chrome".to_string(),
                    device: "".to_string(),
                    system_locale: "en-GB".to_string(),
                    browser_user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36".to_string(),
                    browser_version: "126.0.0.0".to_string(),
                    os_version: "10.15.7".to_string(),
                    referrer: "".to_string(),
                    referring_domain: "".to_string(),
                    referrer_current: "".to_string(),
                    referring_domain_current: "".to_string(),
                    release_channel: "stable".to_string(),
                    client_build_number: 313070,
                    client_event_source: None,
                },
                presence: Presence {
                    status: "invisible".to_string(),
                    since: 0,
                    activities: vec![],
                    afk: true,
                },
                compress: false,
                client_state: ClientState {
                    guild_versions: std::collections::HashMap::new(),
                },
            },
        }
    }
}
