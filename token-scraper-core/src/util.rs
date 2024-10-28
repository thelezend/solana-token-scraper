//! Utility functions for the solana-token-scraper program.

use std::{
    fs::OpenOptions,
    io::{BufRead, Write},
    path::Path,
    str::FromStr,
};

use regex::Regex;
use solana_sdk::pubkey::Pubkey;

/// The base URL for the Jupiter price API.
pub const JUPITER_PRICE_API_URL: &str = "https://price.jup.ag/v6/price";

/// Checks if the provided token address is valid.
///
/// # Arguments
///
/// * `token_address` - A string slice that holds the token address.
///
/// # Errors
///
/// This function does not return errors.
pub fn is_valid_token_address(token_address: &str) -> bool {
    let address = Pubkey::try_from(token_address);
    address.is_ok()
}

/// Sends a sniper request to the specified URL.
///
/// This function sends a GET request to the given URL with the token as a query parameter.
///
/// # Arguments
///
/// * `token` - A string slice that holds the token.
/// * `url` - The base URL to send the request to.
///
/// # Errors
///
/// Returns a `reqwest::Error` if the request fails.
pub async fn send_token_request(token: &str, url: &str) -> Result<(), reqwest::Error> {
    let url = format!("{url}?token={token}");
    reqwest::get(url).await?;
    Ok(())
}

/// Checks if a token address is already detected in a specified file.
///
/// This function reads the specified file line by line to check if the given token address is present.
///
/// # Arguments
///
/// * `token_address` - A string slice that holds the token address.
/// * `file_path` - A reference to the path of the file to be checked.
///
/// # Errors
///
/// Returns a `std::io::Error` if the file operation fails.
pub async fn is_token_already_detected(
    token_address: &str,
    file_path: &Path,
) -> Result<bool, std::io::Error> {
    let file = OpenOptions::new().read(true).open(file_path)?;
    let reader = std::io::BufReader::new(file);

    for line in reader.lines() {
        if line?.trim() == token_address {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Adds a token address to a specified file.
///
/// This function appends the given token address to the specified file.
///
/// # Arguments
///
/// * `token_address` - A string slice that holds the token address.
/// * `file_path` - A reference to the path of the file where the token address will be added.
///
/// # Errors
///
/// Returns a `std::io::Error` if the file operation fails.
pub async fn add_token_to_file(
    token_address: &str,
    file_path: &Path,
) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new().append(true).open(file_path)?;
    writeln!(file, "{}", token_address)?;

    Ok(())
}

/// Checks if the provided link is a Pump.fun link.
///
/// # Arguments
///
/// * `link` - A string slice that holds the link to be checked.
///
/// # Returns
///
/// * `true` if the link contains "https://pump.fun/", otherwise `false`.
pub fn is_pumpfun_link(link: &str) -> bool {
    link.contains("https://pump.fun/")
}

/// Errors that can occur when extracting a token from a Pump.fun link.
#[derive(Debug, thiserror::Error)]
pub enum ExtractTokenFromPumpFunLinkError {
    /// The provided link is invalid.
    #[error("Invalid link")]
    InvalidLink(String),

    /// Failed to parse the token to a Pubkey.
    #[error("Failed to parse token to Pubkey")]
    ParseToken(#[from] solana_sdk::pubkey::ParsePubkeyError),
}

/// Extracts the token from a given Pump.fun link.
///
/// # Arguments
///
/// * `link` - A string slice that holds the Pump.fun link.
///
/// # Returns
///
/// * `Result<Pubkey, ExtractTokenErrorFromPumpFunLink>` containing the token if found, otherwise an error.
pub fn extract_token_from_pumpfun_link(
    link: &str,
) -> Result<Pubkey, ExtractTokenFromPumpFunLinkError> {
    let re = Regex::new(r#"https://pump\.fun/([A-Za-z0-9]+)"#).unwrap();
    let token = re.captures(link).and_then(|caps| caps.get(1)).ok_or(
        ExtractTokenFromPumpFunLinkError::InvalidLink(link.to_string()),
    )?;

    Ok(Pubkey::from_str(token.as_str())?)
}

/// Errors that can occur when fetching the price from the Jupiter API.
///
/// This enum represents the possible errors that can occur during the process of fetching
/// the token price from the Jupiter API.
#[derive(Debug, thiserror::Error)]
pub enum JupPriceApiError {
    /// Error from the `reqwest` crate.
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// Error from the `serde_json` crate.
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// Error during conversion of the JSON value.
    #[error("Conversion error")]
    ConversionError(serde_json::Value),

    /// Error when the price is not found in the response.
    #[error("Price not found in response: {0:?}")]
    PriceNotFound(serde_json::Value),
}

/// Fetches the price of a token from the Jupiter API.
///
/// This function sends a request to the Jupiter API to get the price of the specified token
/// in terms of another token (vs_token).
///
/// # Arguments
///
/// * `token` - A string slice that holds the token address or symbol.
/// * `vs_token` - A string slice that holds the address or symbol of the token to compare against.
///
/// # Returns
///
/// Returns a `Result<f64, JupPriceApiError>` where the `f64` is the price of the token.
///
/// # Errors
///
/// Returns a `JupPriceApiError` if:
/// - The request to the Jupiter API fails.
/// - The response cannot be parsed as JSON.
/// - The price information is not found in the response.
/// ```
pub async fn get_token_price_jup(token: &str, vs_token: &str) -> Result<f64, JupPriceApiError> {
    // Construct the URL for the Jupiter API request
    let url = format!(
        "{}?ids={}&vsToken={}",
        JUPITER_PRICE_API_URL, token, vs_token
    );

    // Send GET request to the Jupiter API and parse the response as JSON
    let resp = reqwest::get(&url)
        .await?
        .json::<serde_json::Value>()
        .await?;

    // Extract the price from the response
    resp.get("data")
        .and_then(|data| data.get(token))
        .and_then(|token_data| token_data.get("price"))
        .ok_or_else(|| JupPriceApiError::PriceNotFound(resp.clone()))
        .and_then(|price| {
            price
                .as_f64()
                .ok_or_else(|| JupPriceApiError::ConversionError(price.clone()))
        })
}

/// Errors that can occur when getting the market cap.
#[derive(Debug, thiserror::Error)]
pub enum MarketCapError {
    /// Error from the `util::get_token_price_jup` function.
    #[error("Failed to get token price from Jupiter API: {0}")]
    GetTokenPriceJup(#[from] JupPriceApiError),
}

/// Fetches the market cap of a token.
///
/// This function calculates the market cap of a token by fetching its price from the Jupiter API
/// and multiplying it by the supply of pumpfun tokens.
///
/// # Arguments
///
/// * `token` - A string slice that holds the token symbol.
///
/// # Errors
///
/// Returns a `MarketCapError` if the request to fetch the token price fails.
pub async fn get_market_cap(token: &str) -> Result<u128, MarketCapError> {
    /// The supply of pumpfun tokens.
    const PUMPFUN_TOKEN_SUPPLY: u128 = 1_000_000_000;

    let price = get_token_price_jup(token, "USDC").await?;
    let market_cap = (price * PUMPFUN_TOKEN_SUPPLY as f64) as u128;
    Ok(market_cap)
}

/// Filters tokens based on their market cap.
///
/// This function fetches the market cap of a token and compares it to a given threshold.
/// If the market cap is below the threshold, it returns `false`. If the market cap is above
/// the threshold or if the token price is not found, it returns `true`.
///
/// # Arguments
///
/// * `token` - A string slice that holds the token symbol.
/// * `mc_threshold` - The market cap threshold.
///
/// # Errors
///
/// Returns a `MarketCapError` if the request to fetch the token price fails.
pub async fn filter_market_cap(token: &str, mc_threshold: u128) -> Result<bool, MarketCapError> {
    let market_cap_res = get_market_cap(token).await;
    match market_cap_res {
        Ok(market_cap) => {
            if mc_threshold <= market_cap {
                Ok(true)
            } else {
                tracing::info!(
                    "Skipping because market cap is above threshold: {}",
                    market_cap
                );
                Ok(false)
            }
        }
        Err(MarketCapError::GetTokenPriceJup(JupPriceApiError::PriceNotFound(resp))) => {
            tracing::debug!("Price not found from Jupiter API: {}", resp);
            tracing::debug!("Proceeding with sniper request");
            Ok(true)
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the `is_pumpfun_link` function.
    #[test]
    fn test_is_pumpfun_link() {
        // Test with a valid Pump.fun link
        assert!(is_pumpfun_link(
            "https://pump.fun/5eMZuRe5JfEz7hdv3ZorNsmcMs4qEGiGH1esFJsJFHka"
        ));
        // Test with a valid Pump.fun link enclosed in quotes
        assert!(is_pumpfun_link(
            "\"https://pump.fun/5eMZuRe5JfEz7hdv3ZorNsmcMs4qEGiGH1esFJsJFHka\""
        ));
        // Test with an invalid Pump.fun link
        assert!(!is_pumpfun_link("pumpfun"));
    }

    /// Tests the `extract_token_from_pumpfun_link` function.
    #[test]
    fn test_extract_token_from_pumpfun_link() {
        // Test with a valid Pump.fun link
        assert_eq!(
            extract_token_from_pumpfun_link(
                "https://pump.fun/5eMZuRe5JfEz7hdv3ZorNsmcMs4qEGiGH1esFJsJFHka"
            )
            .unwrap(),
            Pubkey::from_str("5eMZuRe5JfEz7hdv3ZorNsmcMs4qEGiGH1esFJsJFHka").unwrap()
        );
        // Test with a valid Pump.fun link enclosed in quotes
        assert_eq!(
            extract_token_from_pumpfun_link(
                "\"https://pump.fun/5eMZuRe5JfEz7hdv3ZorNsmcMs4qEGiGH1esFJsJFHka\""
            )
            .unwrap(),
            Pubkey::from_str("5eMZuRe5JfEz7hdv3ZorNsmcMs4qEGiGH1esFJsJFHka").unwrap()
        );
    }

    /// Tests the `get_token_price_jup` function.
    #[tokio::test]
    async fn test_get_token_price_jup() {
        // Test with a valid token and vs_token
        let price =
            get_token_price_jup("CT6sgK6Yz6LyfnSnY3PhS2VdvD2tFYkazPrNZEhNpump", "USDC").await;

        assert!(price.is_ok(), "{}", price.err().unwrap());
    }
}
