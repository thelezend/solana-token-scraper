//! Telegram errors.

use crate::{message_handler::ExtractTokenError, util::MarketCapError};

/// Errors that can occur in the Telegram module.
///
/// This enum represents the possible errors that can occur while interacting with Telegram.
#[derive(Debug, thiserror::Error)]
pub enum TelegramError {
    /// Error when connecting to Telegram.
    #[error("Failed to connect to Telegram: {0}")]
    TelegramConnection(#[from] TelegramConnectionError),

    /// Error when authorizing the client.
    #[error("Failed to authorize: {0}")]
    Authorization(#[from] AuthorizationError),

    /// Error when handling an update.
    #[error("Failed to handle update: {0}")]
    UpdateHandling(#[from] grammers_client::InvocationError),

    /// Error when extracting a token.
    #[error("Failed to extract token: {0}")]
    TokenExtraction(#[from] ExtractTokenError),

    /// Error when adding a token to file.
    #[error("Failed to add token to file: {0}")]
    AddTokenToFile(#[from] std::io::Error),

    /// Error when sending a token request.
    #[error("Failed to send token request: {0}")]
    SendTokenRequest(#[from] reqwest::Error),

    /// Error when filtering market cap.
    #[error("Failed to filter market cap: {0}")]
    FilterMarketCap(#[from] MarketCapError),
}

/// Error types for Telegram connection.
#[derive(Debug, thiserror::Error)]
pub enum TelegramConnectionError {
    /// Error when loading or creating the session file.
    #[error("Failed to load or create session file: {0}")]
    SessionFile(#[from] std::io::Error),

    /// Error when authorizing the client.
    #[error("Failed to authorize: {0}")]
    Authorization(#[from] grammers_client::client::auth::AuthorizationError),
}

/// Error types for prompt input.
#[derive(Debug, thiserror::Error)]
pub enum PromptError {
    /// Error when reading a line from standard input.
    #[error("Failed to read line: {0}")]
    ReadLine(#[from] std::io::Error),
}

/// Error types for authorization.
#[derive(Debug, thiserror::Error)]
pub enum AuthorizationError {
    /// Error when prompting the user for input.
    #[error("Failed to prompt user for input: {0}")]
    Prompt(#[from] PromptError),

    /// Error when invoking a Grammers client method.
    #[error("Failed to invoke Grammers client method: {0}")]
    GrammersInvocation(#[from] grammers_client::InvocationError),

    /// Error when authorizing the client with phone number.
    #[error("Failed to authorize with phone number: {0}")]
    PhoneAuthorization(#[from] grammers_client::client::auth::AuthorizationError),

    /// Error when authorizing the client with a password.
    #[error("Failed to authorize with password: {0}")]
    PasswordAuthorization(#[from] Box<grammers_client::client::auth::SignInError>),
}
