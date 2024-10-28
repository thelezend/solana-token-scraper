//! Telegram utility functions.

use std::io::{BufRead, Write};

use grammers_client::{session::Session, InitParams, SignInError};

use crate::filters::Filter;

use super::{errors::*, retry::RetryPolicy};

/// Connects to the Telegram client.
///
/// This function establishes a connection to the Telegram client using the provided
/// API ID, API hash, and session file.
///
/// # Arguments
///
/// * `api_id` - The API ID for the Telegram client.
/// * `api_hash` - The API hash for the Telegram client.
/// * `session_file` - The path to the session file.
/// ```
pub async fn connect_to_telegram(
    api_id: i32,
    api_hash: String,
    session_file: &str,
) -> Result<grammers_client::Client, TelegramConnectionError> {
    Ok(grammers_client::Client::connect(grammers_client::Config {
        session: Session::load_file_or_create(session_file)?,
        api_id,
        api_hash: api_hash.clone(),
        params: InitParams {
            reconnection_policy: &RetryPolicy { attempts: 3 },
            ..Default::default()
        },
    })
    .await?)
}

/// Prompts the user for input.
///
/// This function displays a message to the user and waits for input from standard input.
///
/// # Arguments
///
/// * `message` - The message to display to the user.
/// ```
pub fn prompt(message: &str) -> Result<String, PromptError> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(message.as_bytes())?;
    stdout.flush()?;

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    Ok(line)
}

/// Authorizes the Telegram client.
///
/// This function checks if the client is authorized. If not, it prompts the user for
/// their phone number, requests a login code, and prompts the user for the received code.
/// If a password is required, it prompts the user for the password. Finally, it saves
/// the session to a file.
///
/// # Arguments
///
/// * `client` - The Telegram client to authorize.
/// * `session_file` - The path to the session file.
/// * `sign_out` - A mutable reference to a boolean indicating whether to sign out.
///
/// # Errors
///
/// This function will return an error if any of the following occurs:
/// * Prompting the user for input fails.
/// * Invoking a Grammers client method fails.
/// * Authorizing the client with phone number fails.
/// * Authorizing the client with a password fails.
pub async fn authorize_client(
    client: &grammers_client::Client,
    session_file: &str,
    sign_out: &mut bool,
) -> Result<(), AuthorizationError> {
    if !client.is_authorized().await? {
        let phone = prompt("Enter your phone number (international format): ")?;
        let token = client.request_login_code(&phone).await?;
        let code = prompt("Enter the code you received: ")?;
        let signed_in = client.sign_in(&token, &code).await;
        match signed_in {
            Err(SignInError::PasswordRequired(password_token)) => {
                // Note: this `prompt` method will echo the password in the console.
                //       Real code might want to use a better way to handle this.
                let hint = password_token.hint().unwrap_or("None");
                let prompt_message = format!("Enter the password (hint {}): ", &hint);
                let password = prompt(prompt_message.as_str())?;

                client
                    .check_password(password_token, password.trim())
                    .await
                    .map_err(Box::new)?;
            }
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        };
        match client.session().save_to_file(session_file) {
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("NOTE: failed to save the session, will sign out when done: {e}");
                *sign_out = true;
            }
        }
    }

    Ok(())
}

/// Filters a message based on the provided Telegram channel ID.
///
/// This function iterates through the provided `filters` and checks if the `channel_id` matches any of the filters' Telegram channel IDs.
/// If a match is found, it returns the matching `Filter`.
///
/// # Arguments
///
/// * `channel_id` - The Telegram channel ID to be checked.
/// * `filters` - A slice of `Filter` objects to be checked against.
///
/// # Errors
///
/// This function does not return any errors.
pub fn filter_message(channel_id: i64, filters: &[Filter]) -> Option<Filter> {
    for filter in filters {
        if let Some(filter_channel_id) = filter.telegram_channel_id {
            if channel_id == filter_channel_id {
                return Some(filter.clone());
            }
        }
    }
    None
}
