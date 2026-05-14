# i-rs-weight

Weight tracking CLI tool for managing weight records with trend visualization.

## Overview

i-rs-weight helps you track your body weight over time with optional ASCII charts and statistics. Great for fitness tracking, health monitoring, and weight management goals.

## Quick Start

```bash
# Add a weight record
i-rs-weight add 2025-01-15 70.5

# Add with remarks
i-rs-weight add 2025-01-16 70.3 --remark "After workout"

# List all records
i-rs-weight list

# List last 30 days with chart and stats
i-rs-weight list --days 30 --chart --stats

# Show only chart
i-rs-weight list --chart

# Show only stats
i-rs-weight list --stats

# Update a record
i-rs-weight update 2025-01-15 --weight 70.0

# Delete a record
i-rs-weight delete 2025-01-15
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-weight

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-weight
```

## Data Storage

- macOS: `~/.config/i-rs/weights.json`
- Linux: `~/.config/i-rs/weights.json`
- Windows: `~\AppData\Roaming\i-rs\weights.json`

## Features

- **Weight Logging**: Record daily weight measurements
- **ASCII Charts**: Visual trend visualization
- **Statistics**: Min, max, average, and change calculations

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records