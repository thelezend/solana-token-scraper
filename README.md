# Solana Token Scraper

[![CI](https://github.com/thelezend/solana-token-scraper/actions/workflows/ci.yml/badge.svg)](https://github.com/thelezend/solana-token-scraper/actions/workflows/ci.yml)

## Introduction

A minimal CLI application designed to detect and scrape Solana token information from social media channels, particularly useful for tracking free and premium alpha group calls.

### Supported platforms

- [x] Discord
- [x] Telegram
- [ ] Twitter

## Working and Usage

The program connects to the Discord gateway using your token and the Telegram API using your credentials, monitoring messages mentioning valid Solana token addresses.

- You can use `filters.csv` to customize it and filter scans to specific channels, groups, and user messages.
- This is not a sniping/trading bot but is designed to work alongside automation/sniping bots like [Peppermints](https://www.tensor.trade/trade/peppermints) or your custom programs with similar functionality.
- Once a token is detected, it will send a GET request to the URL specified in the `TOKEN_ENDPOINT_URL` field.
- The detected token addresses are saved in a local text file `detected_tokens.txt` to avoid duplicate purchases.
- Logs are saved in `logs` directory for debugging purposes.

> **IMPORTANT: Using user accounts for automation is against Discord's TOS, so use them at your own risk, preferably with accounts you can afford to lose. Telegram is more lenient.**

## Installation

You can directly download the pre-built binary executable for your OS and architecture from the [releases](https://github.com/thelezend/solana-token-scraper/releases) page. If you prefer to verify the code, build it yourself, or need to use it on an OS without a binary release, you can easily build it from source. Plenty of Rust-related resources are available online to guide you through the process.

## Configuration

### Settings

Need to have a `settings.json` file in the working directory with the following:

```json
{
    "discord": {
        "user_token": "YOUR_DISCORD_USER_TOKEN",
        "sec_ws_key": "YOUR_DISCORD_SECRET_WS_KEY"
    },
    "telegram": {
        "api_id": "YOUR_TELEGRAM_API_ID",
        "api_hash": "YOUR_TELEGRAM_API_HASH"
    },
    "solana": {
        "rpc_url": "YOUR_RPC_URL"
    }
}
```

- Read how to get a Discord user account token [here](https://gist.github.com/MarvNC/e601f3603df22f36ebd3102c501116c6).
- You can similarly obtain the `Sec-Websocket-Key` from the headers of the WebSocket request to the Discord gateway.
- Telegram's `api_id` and `api_hash` can be obtained from [my.telegram.org](https://my.telegram.org).
- You will be asked to enter your phone number to receive a code to authenticate your Telegram account for the first time. Your session will be saved in `scraper.session`, so this won't be needed every time.
- `rpc_url` is only used for getting token addresses from links. Won't be used to send transactions.

### Filters

Need to have a `filters.csv` file in the working directory with the following:

```csv
NAME,DISCORD_CHANNEL_ID,DISCORD_USER_ID,TELEGRAM_CHANNEL_ID,TOKEN_ENDPOINT_URL,MARKET_CAP
test,12314,123234,,http://localhost:9001/solana,
pow-calls,132414,51451345,,http://localhost:9005/solana,20000
```

- `NAME`(Required): The name of the filter. This is just for your reference.
- `DISCORD_CHANNEL_ID`(Optional): The ID of the Discord channel you want to monitor.
- `DISCORD_USER_ID`(Optional): The ID of the Discord user you wish to monitor.
- `TELEGRAM_CHANNEL_ID`(Optional):The ID of the Telegram channel you wish to monitor. For private Telegram channels, which start with `-100`, ensure that the `-100` is removed from the ID. If you’re unsure how to find a Telegram channel ID, a quick online search can guide you.
- `TOKEN_ENDPOINT_URL`(Required): The URL to which a GET request will be made, with the token address as a parameter.
- `MARKET_CAP`(Optional): Represents the minimum market capitalization required for the token to be detected. The token’s price is retrieved via Jupiter’s API; however, this may not be applicable for very new tokens.

Discord fields are combined using an AND operation, whereas Discord and Telegram fields are combined using an OR operation. For example, in the filter below:

```csv
NAME,DISCORD_CHANNEL_ID,DISCORD_USER_ID,TELEGRAM_CHANNEL_ID,TOKEN_ENDPOINT_URL,MARKET_CAP
test,12314,123234,2254310975,http://localhost:9001/solana,
```

The token is detected if it is posted in the `DISCORD_CHANNEL_ID` of `12314` by the `DISCORD_USER_ID` `123234`, OR if it is posted by anyone in the `TELEGRAM_CHANNEL_ID` `2254310975`.

**Note:** Currently, filtering by USER ID is not supported for Telegram.

## Support and Contact

Feel free to customize and integrate the code as you like. If this has been helpful or profitable, and you’re feeling generous enough to pay for my gym subscription 😅, you can send Solana or any other token to my Solana wallet: `lezend.sol`

If you’d like me to build any app or tool, feel free to reach out to me on Discord.

## Code contributions

Your contributions are welcome! I would love to make this tool better or add new features. Please ensure your code follows the existing style and includes documentation and tests for any new functionality.

For major changes, please open an issue first to discuss what you would like to change or feel free reach out to me on my socials, preferably Discord.

## License

This project is licensed under the MIT OR Apache-2.0 license.
