# i-rs-event

Social event management CLI tool for tracking meetings, gatherings, courses, and other events.

## Features

- Event management (add, list, get, delete)
- Event types: meeting, gathering, course, other
- Tag support for categorization
- Participant tracking
- Location information
- Yearly statistics
- JSON output support

## Install

```bash
cargo build -p i-rs-event
```

## Quick Start

```bash
# Add an event
i-rs-event add "Team Meeting" --date 2024-03-15 --type meeting --location "Conference Room" -p "Alice,Bob" -t work

# List all events
i-rs-event list

# Get event details
i-rs-event get "Team Meeting"

# View statistics
i-rs-event stats --year 2024

# Delete an event
i-rs-event delete "Team Meeting"
```

## Data Storage

- macOS: `~/.config/i-rs/event.json`
- Linux: `~/.config/i-rs/event.json`
- Windows: `~\AppData\Roaming\i-rs\event.json`

## Event Types

| Type | Description |
|------|-------------|
| meeting | Business or team meetings |
| gathering | Social gatherings, parties |
| course | Courses, workshops, classes |
| other | Other events |

## License

MIT OR Apache-2.0
