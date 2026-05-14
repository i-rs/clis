# i-rs-height

Height tracking CLI tool for monitoring body height and weight over time.

## Overview

i-rs-height is a cross-platform CLI tool that helps you track your body measurements including height and weight. It provides ASCII charts and statistics to visualize your progress over time.

## Features

- **Height Recording**: Record height measurements with timestamps
- **Weight Tracking**: Optional weight recording with each height entry
- **ASCII Charts**: Visual representation of height trends
- **Statistics**: Min, max, average, and total change calculations
- **Target Goals**: Set and track progress toward target height
- **Tags & Remarks**: Organize records with tags and notes
- **JSON Output**: Machine-readable output for scripting

## Quick Start

```bash
# Add a height record
i-rs-height add 2025-06-14 175.5

# View your history
i-rs-height list

# See trends with chart
i-rs-height list --chart
```

## Data Storage

All data is stored locally in JSON format:
- macOS: `~/.config/i-rs/heights.json`
- Linux: `~/.config/i-rs/heights.json`
- Windows: `~\AppData\Roaming\i-rs\heights.json`
