# i-rs-habit

Habit tracking CLI tool for building good habits with checkins and streaks.

## Overview

i-rs-habit helps you track and maintain daily habits. It supports:
- Habit creation with frequency settings
- Daily checkins
- Streak calculation
- Tag-based filtering

## Quick Start

```bash
# Create a new habit
i-rs-habit add daily_walk --description "Walk 30 minutes" --frequency daily --tag health

# Checkin for today
i-rs-habit checkin daily_walk

# View all habits with streaks
i-rs-habit list

# Get detailed information
i-rs-habit get daily_walk
```

## Key Features

- **Streak Tracking**: Automatically calculates current streak for each habit
- **Frequency Settings**: Support for daily, weekly, monthly, yearly frequencies
- **Tag Support**: Organize habits with tags
- **JSON Output**: Use `--json` flag for programmatic access