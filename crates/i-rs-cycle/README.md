# i-rs-cycle

Menstrual cycle tracking CLI tool for recording period, spotting, and symptoms.

## Features

- Track different event types (period, spotting, ovulation, etc.)
- Record symptoms
- Date-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-cycle
# or
brew install i-rs/homebrew-tap/i-rs-cycle
```

## Quick Start

```bash
# Record cycle event
i-rs-cycle add 2024-01-15 period
i-rs-cycle add 2024-01-20 spotting --symptom cramps

# List all records
i-rs-cycle list

# Get record details
i-rs-cycle get abc12345
```

## Event Types

- `period` - Menstrual period
- `spotting` - Light bleeding
- `ovulation` - Ovulation day
- `fertile` - Fertile window

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/cycles.json`
- Linux: `~/.config/i-rs/cycles.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0