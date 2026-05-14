# i-rs-ac

AC cleaning tracking CLI tool for recording when you clean air conditioners.

## Features

- Track different AC locations
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-ac
# or
brew install i-rs/homebrew-tap/i-rs-ac
```

## Quick Start

```bash
# Record AC cleaning
i-rs-ac add "Living Room"
i-rs-ac add "Bedroom" --tag summer

# List all records
i-rs-ac list

# Get record details
i-rs-ac get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/ac.json`
- Linux: `~/.config/i-rs/ac.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0