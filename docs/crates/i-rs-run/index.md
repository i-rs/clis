# i-rs-run

Running record CLI tool for tracking running activities with detailed metrics.

## Features

- **Run Records**: Track distance, duration, pace, and heart rate
- **Weather Tracking**: Record weather conditions for each run
- **Statistics**: View cumulative stats (total distance, duration, average pace)
- **Training Plans**: Create and manage running plans with schedules
- **Tags & Remarks**: Organize runs with custom tags and notes
- **JSON Output**: Machine-readable output for automation

## Quick Start

```bash
# Add your first run
i-rs-run add 2025-06-14 5.0 30

# View all runs
i-rs-run list

# Check your statistics
i-rs-run stats
```

## Data Storage

All data is stored locally in JSON format:
- macOS: `~/.config/i-rs/runs.json`
- Linux: `~/.config/i-rs/runs.json`
- Windows: `~\AppData\Roaming\i-rs\runs.json`
