# i-rs-exercise

Exercise record tracking CLI tool for logging and managing fitness activities.

## Overview

i-rs-exercise helps you track daily exercise records with support for multiple exercise types, calorie tracking, tag management, and statistical analysis. Suitable for fitness enthusiasts, athletes, and anyone wanting to log their workout data.

## Quick Start

```bash
# Add an exercise record
i-rs-exercise add "Morning Run" running 30 -c 300

# List all records
i-rs-exercise list

# Filter by tag
i-rs-exercise list --tag cardio

# View statistics
i-rs-exercise stats
```

## Features

- Exercise record management (add, view, update, delete)
- Multiple exercise types (running, swimming, gym, yoga, cycling, etc.)
- Calorie tracking
- Tag-based organization
- Statistical analysis (total duration, calories, type breakdown)
- JSON output support

## Data Storage

- macOS: `~/.config/i-rs/exercises.json`
- Linux: `~/.config/i-rs/exercises.json`
- Windows: `~\AppData\Roaming\i-rs\exercises.json`

## License

MIT OR Apache-2.0
