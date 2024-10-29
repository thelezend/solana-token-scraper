//! Retry policy for Telegram reconnections.

use grammers_client::ReconnectionPolicy;

/// Retry policy for Telegram reconnections.
pub struct RetryPolicy;

impl ReconnectionPolicy for RetryPolicy {
    /// Determines whether to retry the connection based on the number of attempts.
    ///
    /// # Arguments
    ///
    /// * `attempts` - The current number of reconnection attempts.
    ///
    /// # Errors
    ///
    /// Logs an error if the maximum number of reconnection attempts is reached.
    fn should_retry(&self, attempts: usize) -> std::ops::ControlFlow<(), std::time::Duration> {
        tracing::debug!("Reconnecting to Telegram... (Attempt {})", attempts);
        std::ops::ControlFlow::Continue(std::time::Duration::from_secs(1))
    }
}
