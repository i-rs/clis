# i-rs-mood

Mood tracking CLI tool for recording and visualizing your daily mood.

## Overview

i-rs-mood helps you track your emotional well-being over time. Record daily moods with optional tags and notes, view statistics, and visualize patterns with a calendar view.

## Quick Start

```bash
# Record today's mood
i-rs-mood add 2025-01-15 good

# Add with emoji
i-rs-mood add 2025-01-16 😊 --tag weekend

# Add with tags and notes
i-rs-mood add 2025-01-17 great --tag work --content "Project completed!"

# List all records
i-rs-mood list

# List last 7 days with calendar
i-rs-mood list --days 7 --calendar

# Update a record
i-rs-mood update 2025-01-15 --mood okay

# Delete a record
i-rs-mood delete 2025-01-15
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
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Mood Recording**: 5-level mood tracking (Great, Good, Okay, Bad, Terrible)
- **Multiple Input Formats**: Use numbers (1-5), words, or emoji
- **Tags & Notes**: Add context to your mood entries
- **Calendar View**: Visual mood calendar
- **Statistics**: Best, worst, and average mood tracking

## Mood Levels

| Level | Word | Emoji |
|-------|------|-------|
| 5 | Great | 😊 |
| 4 | Good | 🙂 |
| 3 | Okay | 😐 |
| 2 | Bad | 😔 |
| 1 | Terrible | 😢 |

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records
