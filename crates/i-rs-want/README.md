# i-rs-want

Wish list CLI tool for tracking things you want to buy or get.

## Features

- Track wanted items
- Record prices and URLs
- Priority levels (low, medium, high)
- Done/pending status

## Install

```bash
npm install -g @i-rs/i-rs-want
# or
brew install i-rs/homebrew-tap/i-rs-want
```

## Quick Start

```bash
# Add wish list items
i-rs-want add "New Headphones" --price 299.99 --priority high
i-rs-want add "Book: Rust Programming" --price 49.99 --priority medium
i-rs-want add "Keyboard" --url "https://example.com/keyboard" --priority high

# List all items
i-rs-want list

# Mark as done
i-rs-want update "New Headphones" --done
```

## Priority Levels

- `low` - Low priority
- `medium` - Medium priority
- `high` - High priority

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/want.json`
- Linux: `~/.config/i-rs/want.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0