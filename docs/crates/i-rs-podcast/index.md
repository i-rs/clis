# i-rs-podcast

Podcast and course tracking CLI tool for managing your audio/video learning content.

## Features

- Track podcasts and courses with title, author, and duration
- Track listening progress (current position in seconds)
- Add notes during listening sessions
- Filter by status (not_started, in_progress, completed)
- Tag support for organization
- Statistics overview with progress percentage
- JSON output support

## Quick Start

```bash
# Add a podcast
i-rs-podcast add "The Daily" --author "NYT" --duration 3600

# Update listening progress
i-rs-podcast listen "The Daily" --position 1800

# List all podcasts
i-rs-podcast list

# Show statistics
i-rs-podcast stats
```

## Status Icons

- `○` Not Started
- `◐` In Progress
- `●` Completed

## Data Storage

Data is stored in `~/.config/i-rs/podcasts.json`.
