# i-rs-recur

Recurring expenses tracking CLI tool for managing fixed expenses like rent and subscriptions.

## Features

- Track recurring expenses
- Multiple frequency options (daily, weekly, monthly, quarterly, yearly)
- Automatic next payment calculation
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-recur
# or
brew install i-rs/homebrew-tap/i-rs-recur
```

## Quick Start

```bash
# Add recurring expense
i-rs-recur add "Netflix" --amount 15.99 --frequency monthly
i-rs-recur add "Rent" --amount 2000 --frequency monthly
i-rs-recur add "Gym" --amount 300 --frequency monthly

# List all recurring expenses
i-rs-recur list

# Get expense details
i-rs-recur get Netflix
```

## Frequency Options

- `daily` - Every day
- `weekly` - Every week
- `monthly` - Every month
- `quarterly` - Every 3 months
- `yearly` - Every year

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/recur.json`
- Linux: `~/.config/i-rs/recur.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0