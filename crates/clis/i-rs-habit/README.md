# i-rs-habit

Habit tracking CLI tool for building good habits with checkins and streaks.

## Features

- Track daily/weekly/monthly habits
- Checkin with automatic streak calculation
- Tag support for categorization
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-habit
# or
brew install i-rs/homebrew-tap/i-rs-habit
```

## Quick Start

```bash
# Create a habit
i-rs-habit add daily_walk --description "Walk 30 minutes" --frequency daily --tag health

# Checkin for today
i-rs-habit checkin daily_walk

# List all habits
i-rs-habit list
```

## Data Storage

- macOS: `~/Library/Application Support/i-rs/habits.json`
- Linux: `~/.config/i-rs/habits.json`
- Windows: `~\AppData\Roaming\i-rs\habits.json`

## License

MIT OR Apache-2.0