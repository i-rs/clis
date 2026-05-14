# i-rs-toothbrush

Toothbrush replacement tracking CLI tool for recording when you replace toothbrushes.

## Features

- Track toothbrush types
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-toothbrush
# or
brew install i-rs/homebrew-tap/i-rs-toothbrush
```

## Quick Start

```bash
# Record toothbrush replacement
i-rs-toothbrush add "Electric"
i-rs-toothbrush add "Manual" --tag travel

# List all records
i-rs-toothbrush list

# Get record details
i-rs-toothbrush get abc12345
```

## Brush Types

- `Electric` - Electric toothbrush head
- `Manual` - Regular manual toothbrush
- `Kids` - Children's toothbrush
- `Interdental` - Interdental brush

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/toothbrushes.json`
- Linux: `~/.config/i-rs/toothbrushes.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0