//! A WebSocket client that manages sending and receiving messages.

use std::path::Path;

use futures_util::{SinkExt, StreamExt};
use tokio::time::{self, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::{
    errors::{SendHeartbeatError, SendLoginError, WsClientError},
    processor::process_message,
    types::{
        DisconnectionEvent, HeartbeatAckEvent, HeartbeatEvent, HelloEvent, LoginEvent,
        MonitorEvent, OpCode, ReadyEvent,
    },
};

/// The sending half of the WebSocket stream.
pub type WsTx = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Message,
>;
/// The receiving half of the WebSocket stream.
pub type WsRx = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;

/// A WebSocket client that manages sending and receiving messages.
pub struct WsClient {
    /// The authentication token.
    token: String,
    /// The sending half of the WebSocket stream.
    streamtx: WsTx,
    /// The receiving half of the WebSocket stream.
    streamrx: WsRx,
    /// The interval at which heartbeat messages are sent.
    heartbeat_interval: Option<u64>,
    /// The token endpoint URL.
    token_endpoint_url: String,
    /// The Solana RPC URL.
    solana_rpc_url: String,
    /// The file path where detected tokens are stored.
    detected_tokens_file_path: String,
}

impl WsClient {
    /// Creates a new `WsClient` by connecting to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL to connect to.
    /// * `token` - The authentication token.
    /// * `token_endpoint_url` - The token endpoint URL.
    /// * `solana_rpc_url` - The Solana RPC URL.
    /// * `detected_tokens_file_path` - The file path where detected tokens are stored.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `WsClient` on success, or a `WsClientError` on failure.
    pub async fn new(
        url: &str,
        token: &str,
        token_endpoint_url: &str,
        solana_rpc_url: &str,
        detected_tokens_file_path: &str,
    ) -> Result<Self, WsClientError> {
        let (ws_stream, _) = connect_async(url).await?;
        let (write, read) = ws_stream.split();
        Ok(Self {
            token: token.to_owned(),
            streamtx: write,
            streamrx: read,
            heartbeat_interval: None,
            token_endpoint_url: token_endpoint_url.to_owned(),
            solana_rpc_url: solana_rpc_url.to_owned(),
            detected_tokens_file_path: detected_tokens_file_path.to_owned(),
        })
    }

    /// Handles incoming WebSocket messages.
    ///
    /// # Arguments
    ///
    /// * `msg` - The incoming WebSocket message.
    async fn handle_message(&mut self, msg: Message) -> Result<(), WsClientError> {
        if let Ok(text) = msg.to_text() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
                if let Some(op) = value.get("op").and_then(|v| v.as_u64()) {
                    match OpCode::from_u64(op) {
                        Some(OpCode::Hello) => self.handle_hello(value).await?,
                        Some(OpCode::HeartbeatAck) => self.handle_heartbeat_ack(value).await,
                        Some(OpCode::Disconnection) => self.handle_disconnection(value).await,
                        Some(OpCode::Ready) => self.handle_ready(value).await,
                        Some(OpCode::Monitor) => self.handle_monitor(value).await,
                        _ => tracing::warn!("Unknown OpCode: {}", op),
                    }
                }
            }
        }
        Ok(())
    }

    /// Handles the `Hello` event.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON value of the event.
    async fn handle_hello(&mut self, value: serde_json::Value) -> Result<(), SendHeartbeatError> {
        if let Ok(event) = serde_json::from_value::<HelloEvent>(value) {
            tracing::debug!("Received Hello event: {:?}", event);
            self.heartbeat_interval = Some(event.heartbeat_interval);
            // Send initial heartbeat event using send_heartbeat function
            self.send_heartbeat().await?;
        }
        Ok(())
    }

    /// Handles the `HeartbeatAck` event.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON value of the event.
    async fn handle_heartbeat_ack(&mut self, value: serde_json::Value) {
        if let Ok(event) = serde_json::from_value::<HeartbeatAckEvent>(value) {
            tracing::debug!("Received Heartbeat ACK event: {:?}", event);
        }
    }

    /// Handles the `Disconnection` event.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON value of the event.
    async fn handle_disconnection(&mut self, value: serde_json::Value) {
        if let Ok(event) = serde_json::from_value::<DisconnectionEvent>(value) {
            tracing::warn!("Received Disconnection event: {:?}", event);
        }
    }

    /// Handles the `Ready` event.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON value of the event.
    async fn handle_ready(&mut self, value: serde_json::Value) {
        if let Ok(event) = serde_json::from_value::<ReadyEvent>(value) {
            tracing::debug!("Received Ready event: {:?}", event);
        }
    }

    /// Handles the `Monitor` event.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON value of the event.
    async fn handle_monitor(&mut self, value: serde_json::Value) {
        if let Ok(event) = serde_json::from_value::<MonitorEvent>(value) {
            process_message(
                &event.d,
                &self.solana_rpc_url,
                Path::new(&self.detected_tokens_file_path),
                &self.token_endpoint_url,
            )
            .await
            .unwrap();
        }
    }

    /// Sends a login event with the provided token.
    ///
    /// # Arguments
    ///
    /// * `token` - The authentication token.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn send_login(&mut self, token: &str) -> Result<(), SendLoginError> {
        let login_event = LoginEvent {
            op: OpCode::Login,
            token: token.to_string(),
        };
        let msg = serde_json::to_string(&login_event).unwrap();
        tracing::debug!("Sending login event: {:?}", login_event);
        self.streamtx.send(Message::Text(msg)).await?;
        Ok(())
    }

    /// Starts the WebSocket client, sending a login event and processing incoming messages.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub async fn start(&mut self) -> Result<(), WsClientError> {
        // Send login event
        let token = self.token.clone();
        self.send_login(&token).await?;

        let mut heartbeat_interval = self
            .heartbeat_interval
            .map(|interval| time::interval(Duration::from_millis(interval)));

        loop {
            tokio::select! {
                // Handle incoming messages
                msg = self.streamrx.next() => {
                    match msg {
                        Some(Ok(msg)) => self.handle_message(msg).await?,
                        Some(Err(e)) => tracing::warn!("Error receiving message: {:?}", e),
                        None => break,
                    }
                },
                // Handle heartbeat ticking
                _ = async {
                    if let Some(ref mut interval) = heartbeat_interval {
                        interval.tick().await;
                    }
                }, if heartbeat_interval.is_some() => {
                    // Send heartbeat only if heartbeat_interval is Some
                    self.send_heartbeat().await?;
                }
            }
        }

        Ok(())
    }

    /// Sends a heartbeat event.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn send_heartbeat(&mut self) -> Result<(), SendHeartbeatError> {
        let heartbeat_event = HeartbeatEvent {
            op: OpCode::Heartbeat,
        };
        let msg = serde_json::to_string(&heartbeat_event).unwrap();
        tracing::debug!("Sending heartbeat event: {:?}", heartbeat_event);
        self.streamtx.send(Message::Text(msg)).await?;
        Ok(())
    }
}
