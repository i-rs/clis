# i-rs-time

Time tracking CLI for work hours (Pomodoro timer).

## Features

- Start/stop timers for tasks
- Record task names and tags
- Track daily and weekly work hours
- Generate work reports
- Tag support for categorization
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-time
# or
brew install i-rs/homebrew-tap/i-rs-time
```

## Quick Start

```bash
# Start a timer
i-rs-time start "Working on project"

# Stop the timer
i-rs-time stop

# View statistics
i-rs-time stats today

# Generate weekly report
i-rs-time report --days 7
```

## Commands

| Command | Description |
|---------|-------------|
| `start` | Start a new timer |
| `stop` | Stop the current timer |
| `list` | List all time entries |
| `stats` | Show daily/weekly statistics |
| `report` | Generate work reports |
| `get` | Get entry details |
| `delete` | Delete an entry |
| `example` | Show usage examples |
| `skill` | View AI skill documentation |

## Data Storage

- macOS: `~/.config/i-rs/time.json`
- Linux: `~/.config/i-rs/time.json`
- Windows: `~\AppData\Roaming\i-rs\time.json`

## License

MIT OR Apache-2.0
