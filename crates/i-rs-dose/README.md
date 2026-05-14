# i-rs-dose

Medicine dosage tracking CLI tool for recording when you take medications.

## Features

- Record medicine name and dosage
- Track dosage units (tablet, ml, mg, etc.)
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-dose
# or
brew install i-rs/homebrew-tap/i-rs-dose
```

## Quick Start

```bash
# Record medicine intake
i-rs-dose add "Vitamin D" --dosage 1000 --unit IU
i-rs-dose add "Ibuprofen" --dosage 400 --unit mg

# List all records
i-rs-dose list

# Get record details
i-rs-dose get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/dose.json`
- Linux: `~/.config/i-rs/dose.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0