# i-rs-cycling

Cycling record tracking CLI tool for recording and managing cycling activities.

## Overview

i-rs-cycling helps you track your cycling workouts by recording distance, duration, elevation gain, and route information. It automatically calculates average speed and provides cumulative statistics.

## Features

- Record cycling activities with date, distance (km), and duration (minutes)
- Track elevation gain for hill climbing
- Add route descriptions for each ride
- Tag-based organization
- Automatic average speed calculation
- View cumulative statistics
- JSON output support for integration

## Quick Start

```bash
# Add a cycling record
i-rs-cycling add 2025-06-14 25.5 60 --elevation 300

# List all records
i-rs-cycling list

# View statistics
i-rs-cycling stats

# Get record details
i-rs-cycling get <uuid>
```

## Commands

| Command | Description |
|---------|-------------|
| `add` | Add a new cycling record |
| `list` | List all cycling records |
| `get` | View record details |
| `update` | Update an existing record |
| `delete` | Delete a record |
| `stats` | View cumulative statistics |
| `example` | Show usage examples |
| `skill` | Show AI skill documentation |
