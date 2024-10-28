//! Telegram module for the token-scraper application.

mod constants;
mod errors;
mod handler;
mod util;

pub use constants::*;
pub use handler::start;
pub use util::{authorize_client, connect_to_telegram};
