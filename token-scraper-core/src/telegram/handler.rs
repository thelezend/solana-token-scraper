use grammers_client::Update;

use crate::telegram::constants::SESSION_FILE;

use super::{errors::*, util::*};

pub async fn start(api_id: i32, api_hash: String) -> Result<(), TelegramError> {
    let client = connect_to_telegram(api_id, api_hash, SESSION_FILE).await?;
    let mut sign_out = false;
    authorize_client(&client, SESSION_FILE, &mut sign_out).await?;

    tracing::info!("Connected to Telegram!");

    loop {
        let update = client.next_update().await?;
        match update {
            Update::NewMessage(message) if !message.outgoing() => {
                println!("Received message: {}", message.text());
                todo!("Implement message handling")
            }
            _ => {}
        }
    }

    Ok(())
}
