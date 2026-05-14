# i-rs-bestby

Item best-by date tracking CLI tool for recording purchase dates and replacement cycles.

## Features

- Track purchase dates and replacement cycles
- Automatic replacement countdown
- Expiration status indicators (OK, SOON, EXPIRED)
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-bestby
# or
brew install i-rs/homebrew-tap/i-rs-bestby
```

## Quick Start

```bash
# Add item with purchase date and cycle
i-rs-bestby add "Milk" --purchase-date 2024-01-01 --cycle-days 7
i-rs-bestby add "Phone Battery" --purchase-date 2023-06-01 --cycle-days 365

# List all items
i-rs-bestby list

# Get item details
i-rs-bestby get Milk
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/bestby.json`
- Linux: `~/.config/i-rs/bestby.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0