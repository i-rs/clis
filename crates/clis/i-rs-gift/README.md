# i-rs-gift

Gift management CLI tool for tracking gifts given and received.

## Features

- Record gifts with type (sent/received), recipient, occasion, and value
- Support for occasion labels (birthday/christmas/anniversary/valentine)
- Gift exchange statistics
- Tag and remark support for organization
- JSON output support

## Install

```bash
cargo build -p i-rs-gift
```

## Quick Start

```bash
# Add a sent gift
i-rs-gift add "Birthday Watch" sent "Mom" birthday 500 2024-12-25 --tag family

# Add a received gift
i-rs-gift add "AirPods" received "Boss" christmas 1200 2024-12-25 --tag work

# List all gifts
i-rs-gift list

# List sent gifts only
i-rs-gift list --type sent

# List gifts by tag
i-rs-gift list --tag family

# View gift statistics
i-rs-gift stats

# Get gift details
i-rs-gift get "Birthday Watch"

# Delete a gift
i-rs-gift delete "Birthday Watch"
```

## Data Storage

- macOS: `~/.config/i-rs/gifts.json`
- Linux: `~/.config/i-rs/gifts.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Commands

| Command | Description |
|---------|-------------|
| `add` | Add a new gift record |
| `list` | List all gifts or filter by type/tag |
| `get` | Get gift details |
| `delete` | Delete a gift |
| `stats` | Show gift statistics |
| `example` | Show usage examples |

## License

MIT OR Apache-2.0
