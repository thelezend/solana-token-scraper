//! Filters module for the token-scraper application.

use std::path::Path;

use serde::Deserialize;

/// Represents a filter for the token-scraper application.
///
/// This struct is used to define various filters that can be applied to the token-scraper application.
/// It includes fields for filtering by Discord channel ID, Discord user ID, Telegram channel ID,
/// token endpoint URL, and market cap.
#[derive(Debug, Deserialize, Clone)]
pub struct Filter {
    /// The name of the filter.
    #[serde(rename = "NAME")]
    pub name: String,
    /// The Discord channel ID to filter.
    #[serde(rename = "DISCORD_CHANNEL_ID")]
    pub discord_channel_id: Option<u64>,
    /// The Discord user ID to filter.
    #[serde(rename = "DISCORD_USER_ID")]
    pub discord_user_id: Option<u64>,
    /// The Telegram channel ID to filter.
    #[serde(rename = "TELEGRAM_CHANNEL_ID")]
    pub telegram_channel_id: Option<i64>,
    /// The token endpoint URL to send to.
    #[serde(rename = "TOKEN_ENDPOINT_URL")]
    pub token_endpoint_url: String,
    /// The market cap to filter.
    #[serde(rename = "MARKET_CAP")]
    pub market_cap: Option<u128>,
}

#[derive(thiserror::Error, Debug)]
pub enum FiltersError {
    /// File error.
    #[error("File error: {0}")]
    File(#[from] std::io::Error),

    /// Deserialization error.
    #[error("Deserialization error: {0}")]
    Deserialize(#[from] csv::Error),
}

/// Reads filters from a CSV file.
///
/// This function opens the specified CSV file, reads its contents, and deserializes each record into a `Filter` struct.
///
/// # Arguments
///
/// * `file_path` - A `&Path` that holds the path to the CSV file.
///
/// # Errors
///
/// This function will return an error if the file cannot be opened or if deserialization fails.
pub fn read_filters_from_csv(file_path: &Path) -> Result<Vec<Filter>, FiltersError> {
    let mut filters = Vec::new();
    let file = std::fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let mut rdr = csv::Reader::from_reader(reader);

    for result in rdr.deserialize() {
        let record: Filter = result?;
        filters.push(record);
    }

    Ok(filters)
}
