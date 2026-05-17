# i-rs-tick

Duration tracking CLI tool for recording time spent on activities, tasks, and projects.

## Features

- Record task duration
- Track start and end times
- Duration formatting (hours, minutes, seconds)
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-tick
# or
brew install i-rs/homebrew-tap/i-rs-tick
```

## Quick Start

```bash
# Record task duration
i-rs-tick add "Meeting" --duration 3600
i-rs-tick add "Coding" --duration 7200 --remark "Feature implementation"

# List all records
i-rs-tick list

# Get record details
i-rs-tick get abc12345
```

## Duration Format

Durations are specified in seconds:
- 3600 seconds = 1 hour
- 7200 seconds = 2 hours
- 300 seconds = 5 minutes

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/tick.json`
- Linux: `~/.config/i-rs/tick.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0