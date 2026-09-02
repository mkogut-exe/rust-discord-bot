# Rust Discord Bot

A Discord bot written in Rust using poise + serenity. Focused on time utilities and simple connection-monitoring features: metric (decimal) time, live clocks for timezones, ping checks that edit the same message, plus a few utility commands. The bot reads its token from a BOT_TOKEN.txt file at the repository root and registers slash commands on startup.

Summary
- Prefix commands: start with `!` (configured as `!` in the code).
- Slash commands: the bot attempts to register application (slash) commands globally on startup.
- Token source: BOT_TOKEN.txt (must contain the bot token only).
- Primary dependencies: serenity, poise, tokio, chrono, chrono-tz.

Features / Commands
- ping
  - Usage: `!ping` or `/ping`
  - Replies with current latency (round-trip ping).

- check_connection
  - Usage: `!check_connection [check_freq_seconds] [user_id] [ping_max_ms]`
  - Example: `!check_connection 60 123456789012345678 200`
  - Edits the same message every check to show measured latency. If latency exceeds `ping_max_ms` and `user_id` is provided, the bot will ping that user with a warning.
  - Stop it with: `!stop_check_connection`

- metric_time
  - Usage: `!metric_time [utc_offset]`
  - Shows the current metric (decimal) time. Defaults to UTC+2 if no offset is provided.

- metric_clock / stop_metric_clock
  - Usage: `!metric_clock [utc_offset]` and `!stop_metric_clock`
  - Starts/stops a live updating metric (decimal) time message that edits in place every metric minute.

- clock / stop_clock
  - Usage: `!clock [IANA_timezone]` and `!stop_clock`
  - Starts/stops a live updating clock for one timezone (default `Europe/Berlin`).
  - Timezone name must be a valid IANA zone (e.g., `America/New_York`).

- get_time
  - Usage: `!get_time [IANA_timezone]`
  - Returns current time for the provided timezone (no live updates).

- timezone_clock / stop_timezone_clock
  - Usage: `!timezone_clock timezone1,timezone2,...` and `!stop_timezone_clock`
  - Starts/stops a live message that shows the time for multiple timezones.

- secure_command
  - Usage: `!secure_command`
  - Example of a command that checks user roles by role ID (`1429573519183843538` is included as an allowed ID in the code). Replace or add role IDs as needed.

- sync_slash
  - Usage: `!sync_slash`
  - Owner-only, hidden in help. Forces global slash command registration (the bot registers globally on startup as well).

How the bot reads its token
- At startup the bot reads the token from a file named `BOT_TOKEN.txt` at the project root.
- The handler is:
  - `fn get_token() -> String { fs::read_to_string("BOT_TOKEN.txt").expect(...) }`
- DO NOT commit this file. Keep it out of source control.

Requirements
- Rust toolchain (rustup) and cargo — tested with the dependencies in Cargo.toml:
  - serenity, poise, tokio, chrono, chrono-tz
- Discord bot created in the Discord Developer Portal with the bot token.
- If you use MESSAGE_CONTENT intent (the code requests it), enable the "Message Content Intent" in the Developer Portal for your bot.

Quickstart — run locally
1. Clone and build
   git clone https://github.com/<your-username>/rust-discord-bot.git
   cd rust-discord-bot

2. Create token file (local dev)
   echo "YOUR_BOT_TOKEN" > BOT_TOKEN.txt
   chmod 600 BOT_TOKEN.txt

3. Run in dev
   cargo run

4. Build a release binary
   cargo build --release
   # Binary at target/release/Rust_discord_bot (name follows Cargo.toml package name)

Notes:
- The bot registers slash commands globally on startup (may take up to an hour to appear globally). You can also run `!sync_slash` (owner-only) to trigger registration again.
