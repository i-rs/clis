# i-rs-mood

Mood tracking CLI tool for recording and visualizing your daily mood.

## Overview

i-rs-mood helps you track your emotional well-being over time. Record daily moods with optional tags and notes, view statistics, and visualize patterns with a calendar view.

## Quick Start

```bash
# Record today's mood
i-rs-mood add good

# Add with emoji for a specific date
i-rs-mood add 😊 --date 2025-01-16 --tag weekend

# Add with tags and remarks
i-rs-mood add great --tag work --remark "Project completed!" --date 2025-01-17

# List all records
i-rs-mood list

# List last 7 days with calendar
i-rs-mood list --days 7 --calendar

# Get a record by id
i-rs-mood get abc12345

# Update a record
i-rs-mood update abc12345 --mood okay

# Delete a record
i-rs-mood delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-mood

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-mood
```

## Data Storage

- macOS: `~/.config/i-rs/moods.json`
- Linux: `~/.config/i-rs/moods.json`
- Windows: `~\AppData\Roaming\i-rs\mood.json`

## Features

- **Mood Recording**: 7-level mood tracking (Amazing, Great, Good, Okay, Poor, Bad, Terrible)
- **Multiple Input Formats**: Use numbers (1-7), words, or emoji
- **Tags & Remarks**: Add context to your mood entries
- **Calendar View**: Visual mood calendar
- **Statistics**: Best, worst, and average mood tracking

## Mood Levels

| Level | Word | Emoji |
|-------|------|-------|
| 7 | Amazing | 🤩 |
| 6 | Great | 😊 |
| 5 | Good | 🙂 |
| 4 | Okay | 😐 |
| 3 | Poor | 😕 |
| 2 | Bad | 😔 |
| 1 | Terrible | 😢 |

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records
