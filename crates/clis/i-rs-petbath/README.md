# i-rs-petbath

Pet bath tracking CLI tool for recording when you bathe your pets.

## Features

- Track pet bathing events
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-petbath
# or
brew install i-rs/homebrew-tap/i-rs-petbath
```

## Quick Start

```bash
# Record pet bath
i-rs-petbath add "Cat"
i-rs-petbath add "Dog" --tag summer

# List all records
i-rs-petbath list

# Get record details
i-rs-petbath get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/petbath.json`
- Linux: `~/.config/i-rs/petbath.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0