# i-rs-event

Social event management CLI tool for tracking meetings, gatherings, courses, and other events.

## Overview

i-rs-event helps you manage social events with support for:
- Multiple event types
- Participant tracking
- Location information
- Tag categorization
- Yearly statistics

## Quick Start

```bash
# Add a meeting
i-rs-event add "Team Meeting" --date 2024-03-15 --type meeting --location "Conference Room A" -p "Alice,Bob" -t work

# List all events
i-rs-event list

# Get event details
i-rs-event get "Team Meeting"

# View statistics
i-rs-event stats
```

## Event Types

| Type | Description |
|------|-------------|
| meeting | Business meetings, team standups |
| gathering | Parties, social events |
| course | Workshops, training, classes |
| other | Miscellaneous events |

## Data Storage

All data is stored locally in JSON format:
- macOS: `~/.config/i-rs/event.json`
- Linux: `~/.config/i-rs/event.json`
- Windows: `~\AppData\Roaming\i-rs\event.json`
