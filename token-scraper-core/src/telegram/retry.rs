//! Retry policy for Telegram reconnections.

use grammers_client::ReconnectionPolicy;

/// Retry policy for Telegram reconnections.
///
/// This struct defines the number of reconnection attempts allowed.
pub struct RetryPolicy {
    /// Maximum number of reconnection attempts.
    pub attempts: u8,
}

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
        if attempts < self.attempts as usize {
            tracing::debug!("Reconnecting to Telegram...");
            std::ops::ControlFlow::Continue(std::time::Duration::from_secs(3))
        } else {
            tracing::error!(
                "Reached maximum number of reconnection attempts ({}) for Telegram!",
                self.attempts
            );
            std::ops::ControlFlow::Break(())
        }
    }
}
