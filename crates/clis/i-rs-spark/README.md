# i-rs-spark

Inspiration and ideas tracking CLI tool for capturing fleeting thoughts and creative sparks.

## Features

- Capture inspiration and ideas
- Record content and source
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-spark
# or
brew install i-rs/homebrew-tap/i-rs-spark
```

## Quick Start

```bash
# Capture inspiration
i-rs-spark add "Use machine learning for text classification"
i-rs-spark add "New app idea" --source "Dream"

# List all sparks
i-rs-spark list

# Get spark details
i-rs-spark get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/spark.json`
- Linux: `~/.config/i-rs/spark.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0