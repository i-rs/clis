# i-rs-run

Running record CLI tool for tracking running activities with detailed metrics.

## Features

- Record running activities (distance, duration, pace, heart rate)
- Track weather conditions and tags
- View cumulative statistics (total distance, duration, average pace)
- Manage running plans with schedules
- Tag support for categorization
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-run
# or
brew install i-rs/homebrew-tap/i-rs-run
```

## Quick Start

```bash
# Add a run record
i-rs-run add 2025-06-14 5.0 30

# List all records
i-rs-run list

# View statistics
i-rs-run stats

# Add a training plan
i-rs-run plan-add "5K Training" 5.0 6:00 --schedule 1 3 5
```

## Commands

### Run Records

- `add <DATE> <DISTANCE> <DURATION>` - Add a run record
- `list` - List all run records
- `get <ID>` - Get run record details
- `delete <ID>` - Delete a run record
- `stats` - Show cumulative statistics

### Run Plans

- `plan-add <NAME> <TARGET> <PACE>` - Add a running plan
- `plan-list` - List all plans
- `plan-get <ID>` - Get plan details
- `plan-delete <ID>` - Delete a plan

## Options

- `--json` - JSON output format
- `--heart-rate, -r` - Heart rate (bpm)
- `--weather, -w` - Weather conditions
- `--tags, -t` - Tags (repeatable)
- `--remark` - Remarks (repeatable)
- `--schedule, -s` - Schedule days (0-6 for Sun-Sat)

## Data Storage

- macOS: `~/.config/i-rs/runs.json`
- Linux: `~/.config/i-rs/runs.json`
- Windows: `~\AppData\Roaming\i-rs\runs.json`

## License

MIT OR Apache-2.0
