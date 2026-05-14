# i-rs-feedpet

Pet feeding tracking CLI tool for recording when you feed your pets.

## Features

- Track different pets and food types
- Record feeding amounts
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-feedpet
# or
brew install i-rs/homebrew-tap/i-rs-feedpet
```

## Quick Start

```bash
# Record pet feeding
i-rs-feedpet add "Cat" dry-food "50g"
i-rs-feedpet add "Dog" wet-food "200g" --tag morning

# List all records
i-rs-feedpet list

# Get record details
i-rs-feedpet get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/feedpet.json`
- Linux: `~/.config/i-rs/feedpet.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0