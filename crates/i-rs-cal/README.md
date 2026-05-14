# i-rs-cal

Calorie tracking CLI tool for estimating and recording calorie intake.

## Features

- Track food items and calorie estimates
- Daily calorie totals
- Tag support for categorization
- Date-based history

## Install

```bash
npm install -g @i-rs/i-rs-cal
# or
brew install i-rs/homebrew-tap/i-rs-cal
```

## Quick Start

```bash
# Record calorie intake
i-rs-cal add "Apple" 95
i-rs-cal add "Pizza" 285 --tag lunch
i-rs-cal add "Burger" 350 --date 2024-01-15

# List all records
i-rs-cal list

# Get record details
i-rs-cal get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/cal.json`
- Linux: `~/.config/i-rs/cal.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0