# i-rs-towel

Towel replacement tracking CLI tool for recording when you replace towels.

## Features

- Track towel types
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-towel
# or
brew install i-rs/homebrew-tap/i-rs-towel
```

## Quick Start

```bash
# Record towel replacement
i-rs-towel add bath
i-rs-towel add face --tag bedroom

# List all records
i-rs-towel list

# Get record details
i-rs-towel get abc12345
```

## Towel Types

- `bath` - Bath towel
- `face` - Face towel
- `hand` - Hand towel
- `beach` - Beach towel
- `sports` - Sports towel

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/towels.json`
- Linux: `~/.config/i-rs/towels.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0