//! Telegram errors.

#[derive(Debug, thiserror::Error)]
pub enum TelegramError {
    #[error("Failed to connect to Telegram: {0}")]
    TelegramConnection(#[from] TelegramConnectionError),

    #[error("Failed to authorize: {0}")]
    Authorization(#[from] AuthorizationError),

    #[error("Failed to handle update: {0}")]
    UpdateHandling(#[from] grammers_client::InvocationError),
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
