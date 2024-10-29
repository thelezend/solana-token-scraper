//! Handles the websocket stream for the discord gateway.

use std::{str::FromStr, sync::Arc, time::Duration};

use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tokio::{
    net::TcpStream,
    sync::{mpsc::UnboundedSender, Mutex},
};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use twilight_model::gateway::event::{DispatchEvent, GatewayEvent};

use crate::discord::stream::{identify, util::send_heartbeat};

use super::util::deserialize_event;

/// Type alias for a thread-safe, asynchronous writer to the WebSocket stream.
pub type WebsocketWrite =
    Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>;

/// Error types for the WebSocket handler.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// WebSocket error.
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
}

/// Handles the WebSocket stream for the Discord gateway.
///
/// This function processes incoming WebSocket messages, handles different types of gateway events,
/// and manages the heartbeat and identification processes.
///
/// # Errors
///
/// This function will return an error if there is a WebSocket error.
pub async fn handle_stream(
    discord_token: &str,
    mut ws_read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ws_write: WebsocketWrite,
    event_tx: Arc<UnboundedSender<DispatchEvent>>,
    sequence: Arc<Mutex<Option<u64>>>,
    resume_gateway_url: Arc<Mutex<Option<String>>>,
    session_id: Arc<Mutex<Option<String>>>,
) -> Result<(), Error> {
    while let Some(message) = ws_read.next().await {
        let message = message?.to_string();
        let event = deserialize_event(&message);
        if event.is_err() {
            if handle_exception(
                &message.clone(),
                Arc::clone(&sequence),
                Arc::clone(&resume_gateway_url),
                Arc::clone(&session_id),
            )
            .await
            .is_err()
            {
                tracing::debug!(
                    "Failed to deserialize event: {}. Error: {:?}",
                    message,
                    event.err()
                );
            }
            continue;
        }

        let sequence = Arc::clone(&sequence);
        let ws_write = Arc::clone(&ws_write);
        let discord_token = discord_token.to_string();
        let event_tx = Arc::clone(&event_tx);

        let task = tokio::spawn(async move {
            match event.unwrap() {
                GatewayEvent::Hello(data) => {
                    tracing::trace!("Received hello event: {:?}", data);

                    tracing::trace!("Sending heartbeat");
                    send_heartbeat(Arc::clone(&sequence), Arc::clone(&ws_write))
                        .await
                        .expect("Failed to send heartbeat");

                    identify::identify(discord_token, Arc::clone(&ws_write))
                        .await
                        .expect("Failed to identify");

                    tokio::spawn(async move {
                        loop {
                            tokio::time::sleep(Duration::from_millis(data.heartbeat_interval))
                                .await;
                            if let Err(e) =
                                send_heartbeat(Arc::clone(&sequence), Arc::clone(&ws_write)).await
                            {
                                tracing::debug!("Failed to send heartbeat: {:?}", e);
                                break;
                            }
                        }
                    });
                }
                GatewayEvent::HeartbeatAck => {}
                GatewayEvent::Heartbeat(data) => {
                    let mut sequence_lock = sequence.lock().await;
                    *sequence_lock = Some(data);
                    drop(sequence_lock);

                    let sequence = Arc::clone(&sequence);
                    let ws_write = Arc::clone(&ws_write);

                    send_heartbeat(sequence, ws_write)
                        .await
                        .expect("Failed to send heartbeat");
                }
                GatewayEvent::Dispatch(seq, dispatch_event) => {
                    let mut sequence_lock = sequence.lock().await;
                    *sequence_lock = Some(seq);
                    drop(sequence_lock);

                    event_tx
                        .send(dispatch_event)
                        .expect("Failed to send dispatch event through channel");
                }
                _ => {}
            }
        })
        .await;

        if let Err(e) = task {
            tracing::error!("Task error: {:?}", e);
        }
    }

    Ok(())
}

/// Error types for handling exceptions.
#[derive(Debug, thiserror::Error)]
pub enum HandleExceptionError {
    /// Error deserializing the exception.
    #[error("Failed to deserialize exception: {0}")]
    Json(#[from] serde_json::Error),
}

/// Handles exceptions received from the Discord WebSocket.
///
/// This function processes the exception message, updates the sequence number,
/// and handles specific events such as "READY".
///
/// # Errors
///
/// This function will return an error if deserializing the message fails.
async fn handle_exception(
    message: &str,
    sequence: Arc<Mutex<Option<u64>>>,
    resume_gateway_url: Arc<Mutex<Option<String>>>,
    session_id: Arc<Mutex<Option<String>>>,
) -> Result<(), HandleExceptionError> {
    let json_value = serde_json::Value::from_str(message)?;

    match json_value.get("t") {
        Some(value) => {
            let mut sequence_lock = sequence.lock().await;
            *sequence_lock = Some(json_value.get("s").unwrap().as_u64().unwrap());
            drop(sequence_lock);

            let event_type = value.as_str().unwrap_or("").trim_matches('"');

            match event_type {
                "READY" => {
                    tracing::debug!("Received ready event");
                    tracing::debug!(
                        "Logged in as {}",
                        json_value
                            .get("d")
                            .unwrap()
                            .get("user")
                            .unwrap()
                            .get("username")
                            .unwrap()
                    );

                    let mut resume_gateway_url_lock = resume_gateway_url.lock().await;
                    *resume_gateway_url_lock = Some(
                        json_value
                            .get("d")
                            .unwrap()
                            .get("resume_gateway_url")
                            .unwrap()
                            .to_string(),
                    );
                    drop(resume_gateway_url_lock);

                    let mut session_id_lock = session_id.lock().await;
                    *session_id_lock = Some(
                        json_value
                            .get("d")
                            .unwrap()
                            .get("session_id")
                            .unwrap()
                            .to_string(),
                    );
                    drop(session_id_lock);
                }
                _ => {
                    tracing::debug!("Received event: {}, skipping", event_type);
                }
            }
        }
        None => {
            tracing::warn!("t is missing from the event: {}", message);
        }
    }

    Ok(())
}
