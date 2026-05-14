# i-rs-ledger

Personal accounting CLI tool for tracking income, expenses, and transfers.

## Features

- Track income, expenses, and transfers
- Category-based organization
- Multi-currency support
- Financial summary statistics

## Install

```bash
npm install -g @i-rs/i-rs-ledger
# or
brew install i-rs/homebrew-tap/i-rs-ledger
```

## Quick Start

```bash
# Add income
i-rs-ledger add --type income --amount 5000 --category salary

# Add expense
i-rs-ledger add --type expense --amount 150 --category food

# Add transfer
i-rs-ledger add --type transfer --amount 1000 --category savings

# List all entries
i-rs-ledger list

# Get entry details
i-rs-ledger get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/ledger.json`
- Linux: `~/.config/i-rs/ledger.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0