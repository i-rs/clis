# i-rs-fast

Fasting tracking CLI tool for recording fasting sessions.

## Features

- Track fasting start time
- Set target fasting hours
- Auto-calculate actual duration
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-fast
# or
brew install i-rs/homebrew-tap/i-rs-fast
```

## Quick Start

```bash
# Start fasting
i-rs-fast add 16
i-rs-fast add 24 --tag omad

# List all records
i-rs-fast list

# Get record details
i-rs-fast get abc12345
```

## Fasting Types

Common fasting protocols:
- `16:8` - 16 hours fasting, 8 hours eating
- `18:6` - 18 hours fasting, 6 hours eating
- `20:4` - 20 hours fasting, 4 hours eating
- `OMAD` - One Meal A Day (23:1)

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/fast.json`
- Linux: `~/.config/i-rs/fast.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0