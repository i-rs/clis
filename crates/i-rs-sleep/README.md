# i-rs-sleep

Sleep tracking CLI tool for recording bedtime, wake time, and sleep quality.

## Features

- Track bedtime and wake time
- Rate sleep quality (1-5)
- View sleep statistics
- Tag support for categorization
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-sleep
# or
brew install i-rs/homebrew-tap/i-rs-sleep
```

## Quick Start

```bash
# Record sleep
i-rs-sleep add 22:30 06:45 4 --tag workday

# List all records
i-rs-sleep list

# View statistics
i-rs-sleep stats
```

## Data Storage

- macOS: `~/Library/Application Support/i-rs/sleep.json`
- Linux: `~/.config/i-rs/sleep.json`
- Windows: `~\AppData\Roaming\i-rs\sleep.json`

## License

MIT OR Apache-2.0