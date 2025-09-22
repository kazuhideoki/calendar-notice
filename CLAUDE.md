# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Calendar Notice is a Rust application that fetches Google Calendar events and provides notifications. It features:
- OAuth2 authentication with Google
- SQLite database for token and event storage
- TUI (Terminal User Interface) using ratatui
- Daemon mode for background operation
- Automatic calendar sync and notifications

## Common Commands

### Build and Run
```bash
# Build the project
cargo build

# Run with TUI (default)
cargo run

# Run in daemon mode (background, no TUI)
cargo run -- --daemon

# Run release build
cargo run --release
```

### Database Setup
```bash
# Install diesel CLI (one-time)
cargo install diesel_cli

# Initialize database
diesel setup

# Run migrations
diesel migration run
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name
```

## Architecture

### Core Components

1. **OAuth Module** (`src/oauth/`)
   - Handles Google OAuth2 flow
   - Spawns redirect server on port 8990
   - Manages token storage and expiration

2. **Google Calendar Module** (`src/google_calendar/`)
   - Fetches calendar events from Google API
   - Extracts meeting links (Zoom, Teams)
   - Runs sync cron job

3. **Notification Module** (`src/notification/`)
   - Filters upcoming events
   - Manages notification timing
   - Runs notification cron job

4. **TUI Module** (`src/tui/`)
   - Terminal interface using ratatui
   - Displays calendar events
   - Allows user interaction

5. **Repository Module** (`src/repository/`)
   - Database models and operations
   - Uses Diesel ORM with SQLite

### Key Design Patterns

- **Async Runtime**: Uses Tokio for async operations
- **Background Jobs**: Spawns separate tasks for OAuth server, calendar sync, and notifications
- **CLI Parsing**: Uses clap for command-line arguments
- **Error Handling**: Currently uses basic error handling (noted as improvement area in comments)

### Database Schema

Two main tables:
1. `oauth_tokens` - Stores Google OAuth tokens
2. `events` - Stores calendar event data

## Configuration

1. **OAuth Setup**: Create `oauth_secret.json` from `oauth_secret_sample.json` with Google OAuth credentials
2. **Environment**: Create `.env` file with `DATABASE_URL=sqlite://db.sqlite`
3. **Google Cloud**: Configure OAuth2 client with redirect URI `http://localhost:8990/auth`

## Testing Approach

Tests are located within module files using `#[cfg(test)]` blocks. Currently found in:
- `src/oauth/oauth_secret.rs`
- `src/oauth/is_token_expired.rs`
- `src/notification/filter_upcoming_events.rs`