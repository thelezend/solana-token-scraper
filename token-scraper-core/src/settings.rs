//! Settings module for the token-scraper application.
//!
//! This module handles loading and managing the configuration settings for the application.
//! It provides a `Settings` struct that holds the configuration values and a method to load these values from a JSON file.

use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;
use thiserror::Error;

/// Path to the settings file.
pub const SETTINGS_FILE_PATH: &str = "settings.json";

/// Error types for settings module.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error.
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Deserialization error.
    #[error(transparent)]
    Deserialize(#[from] serde_json::Error),
}

/// Settings for the token-scraper application.
///
/// This struct holds the configuration settings for the application.
#[derive(Debug, Deserialize)]
pub struct Settings {
    /// Discord settings.
    pub discord: DiscordSettings,
    /// Solana settings.
    pub solana: SolanaSettings,
}

/// Discord settings for the token-scraper application.
#[derive(Debug, Deserialize)]
pub struct DiscordSettings {
    /// Token for the Discord user.
    pub user_token: String,
    /// Secret WebSocket key.
    pub sec_ws_key: String,
}

/// Solana settings for the token-scraper application.
#[derive(Debug, Deserialize)]
pub struct SolanaSettings {
    /// RPC URL for the Solana network.
    pub rpc_url: String,
}

impl Settings {
    /// Creates a new instance of `Settings` by loading the configuration from `settings.json`.
    ///
    /// This function reads the `settings.json` file, parses it, and deserializes it into a `Settings` struct.
    ///
    /// # Errors
    ///
    /// This function will return an error if the configuration file cannot be read or if the deserialization fails.
    pub fn new() -> Result<Self, Error> {
        let config = Config::builder()
            .add_source(File::new(SETTINGS_FILE_PATH, FileFormat::Json))
            .build()?;

        let settings = config.try_deserialize()?;
        Ok(settings)
    }
}
