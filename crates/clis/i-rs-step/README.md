# i-rs-step

Step counting CLI tool for tracking daily walking and running steps.

## Features

- Record daily step count
- Track distance (optional)
- Time-based history
- Statistics summary

## Install

```bash
npm install -g @i-rs/i-rs-step
# or
brew install i-rs/homebrew-tap/i-rs-step
```

## Quick Start

```bash
# Record steps
i-rs-step add 10000
i-rs-step add 8000 --distance 6.4

# List all records
i-rs-step list

# Get record details
i-rs-step get 2024-01-15
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/step.json`
- Linux: `~/.config/i-rs/step.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0