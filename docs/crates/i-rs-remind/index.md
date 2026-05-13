# i-rs-remind

Reminder management CLI tool for managing events and reminders.

## Overview

i-rs-remind helps you track events, appointments, and reminders with dates. Perfect for birthdays, meetings, deadlines, and recurring events.

## Quick Start

```bash
# Add a reminder
i-rs-remind add meeting 2025-06-15 14:00 --title "Team Meeting" --tag work --content "Discuss project进展"

# Add birthday reminder
i-rs-remind add birthday 2025-08-20 --title "Friend's Birthday" --tag personal

# List reminders
i-rs-remind list

# List by tag
i-rs-remind list --tag work

# Get reminder details
i-rs-remind get meeting

# Mark as done
i-rs-remind done meeting

# Update reminder
i-rs-remind update meeting --content "New agenda items"

# Delete reminder
i-rs-remind delete meeting
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-remind

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-remind
```

## Data Storage

- macOS: `~/.config/i-rs/reminds.json`
- Linux: `~/.config/i-rs/reminds.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records