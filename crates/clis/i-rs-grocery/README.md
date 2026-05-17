# i-rs-grocery

Grocery list CLI tool for managing shopping lists with quantities and purchase tracking.

## Features

- Add items with quantity and unit
- Mark items as purchased/needed
- Filter items by tag or status
- Clear purchased items
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-grocery
# or
brew install i-rs/homebrew-tap/i-rs-grocery
```

## Quick Start

```bash
# Add items
i-rs-grocery add milk 2 bottles --tag dairy
i-rs-grocery add eggs 1 dozen --tag dairy

# List items
i-rs-grocery list

# Mark as purchased
i-rs-grocery purchase milk

# Clear purchased items
i-rs-grocery clear
```

## Data Storage

- macOS: `~/Library/Application Support/i-rs/grocery.json`
- Linux: `~/.config/i-rs/grocery.json`
- Windows: `~\AppData\Roaming\i-rs\grocery.json`

## License

MIT OR Apache-2.0