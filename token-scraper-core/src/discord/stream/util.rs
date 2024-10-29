//! Utility functions for the Discord stream.
use std::sync::Arc;

use futures_util::SinkExt;
use serde::de::DeserializeSeed;
use serde_json::Deserializer;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite;
use twilight_model::gateway::{
    event::{GatewayEvent, GatewayEventDeserializer},
    payload,
};

use super::handler::WebsocketWrite;

/// Error types for deserializing events.
///
/// This enum represents the possible errors that can occur while deserializing
/// events from the Discord WebSocket stream.
#[derive(Debug, thiserror::Error)]
pub enum DeserializeEventError {
    /// Error deserializing the JSON message.
    #[error("Deserialize error: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// Error with the GatewayEventDeserializer.
    #[error("GatewayEventDeserializer error: {0}")]
    GatewayEventDeserializer(String),
}

/// Deserializes a JSON message into a `GatewayEvent`.
///
/// This function takes a JSON message as input and attempts to deserialize it
/// into a `GatewayEvent` using the `GatewayEventDeserializer`.
///
/// # Errors
///
/// This function will return an error if the deserialization process fails.
pub fn deserialize_event(message: &str) -> Result<GatewayEvent, DeserializeEventError> {
    let deserializer = GatewayEventDeserializer::from_json(message);

    match deserializer {
        Some(deserializer) => {
            let mut json_deserializer = Deserializer::from_str(message);
            let event: GatewayEvent = deserializer.deserialize(&mut json_deserializer)?;

            Ok(event)
        }
        None => Err(DeserializeEventError::GatewayEventDeserializer(
            message.to_string(),
        )),
    }
}

/// Error types for sending heartbeats.
///
/// This enum represents the possible errors that can occur while sending
/// heartbeats to the Discord WebSocket stream.
#[derive(Debug, thiserror::Error)]
pub enum HeartbeatError {
    /// Error deserializing the JSON message.
    #[error("Deserialize error: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// WebSocket error.
    #[error("Websocket error: {0}")]
    Ws(tokio_tungstenite::tungstenite::Error),
}

/// Sends a heartbeat to the Discord WebSocket.
///
/// This function sends a heartbeat message to the Discord WebSocket to keep
/// the connection alive. It uses the provided sequence number and WebSocket
/// writer to send the heartbeat.
///
/// # Errors
///
/// This function will return an error if the deserialization process fails or
/// if there is a WebSocket error.
pub async fn send_heartbeat(
    sequence: Arc<Mutex<Option<u64>>>,
    ws_write: WebsocketWrite,
) -> Result<(), HeartbeatError> {
    let sequence = sequence.lock().await;
    let payload = payload::outgoing::Heartbeat::new(*sequence);
    let mut ws_write = ws_write.lock().await;
    ws_write
        .send(tungstenite::Message::from(serde_json::to_string(&payload)?))
        .await
        .map_err(HeartbeatError::Ws)?;

    tracing::trace!("Sent heartbeat: {:?}", payload);

    Ok(())
}
