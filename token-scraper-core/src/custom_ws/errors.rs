//! Custom errors for the WebSocket client.

/// Errors that can occur in the WebSocket client.
#[derive(Debug, thiserror::Error)]
pub enum WsClientError {
    /// Error that occurs when failing to connect to the WebSocket.
    #[error("Websocket client error: {0}")]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),
    /// Error that occurs when failing to send a heartbear message.
    #[error(transparent)]
    SendHeartbeat(#[from] SendHeartbeatError),
    /// Error that occurs when failing to send a login message.
    #[error(transparent)]
    SendLogin(#[from] SendLoginError),
    /// Error that occurs when failing to extract a token from a message.
    #[error(transparent)]
    ExtractToken(#[from] crate::message_handler::ExtractTokenError),
    /// Error that occurs when failing to check if a token has already been detected.
    #[error("Failed to check if token has already been detected: {0}")]
    IsTokenAlreadyDetected(#[from] std::io::Error),
    /// Error that occurs when failing to send a token request.
    #[error("Failed to send token request: {0}")]
    SendTokenRequest(#[from] reqwest::Error),
}

/// Errors that can occur when sending a heartbeat message.
#[derive(Debug, thiserror::Error)]
#[error("Failed to send heartbeat message: {0}")]
pub struct SendHeartbeatError(#[from] tokio_tungstenite::tungstenite::Error);

/// Errors that can occur when sending a login message.
#[derive(Debug, thiserror::Error)]
#[error("Failed to send login message: {0}")]
pub struct SendLoginError(#[from] tokio_tungstenite::tungstenite::Error);
