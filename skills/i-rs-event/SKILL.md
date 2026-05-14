---
name: "i-rs-event"
description: "Social event management CLI tool. Invoke when managing meetings, gatherings, courses, or other events."
---

# i-rs-event

Social event management CLI tool for tracking meetings, gatherings, courses, and other events.

## Storage

- Config: `~/.config/i-rs/event.json`

## Commands

### add

Add a new event.

```bash
i-rs-event add <name> --date <DATE> [OPTIONS]

Options:
  -d, --date <DATE>           Event date (YYYY-MM-DD or YYYY-MM-DD HH:MM)
  -t, --type <TYPE>           Event type (meeting, gathering, course, other)
  -l, --location <LOCATION>   Event location
  -p, --participants <P>      Participants (comma-separated)
  -g, --tags <TAGS>           Tags (comma-separated)
  -r, --remark <REMARK>       Remarks (can be specified multiple times)
```

### list

List all events.

```bash
i-rs-event list [OPTIONS]

Options:
  -t, --tag <TAG>        Filter by tag
  -y, --type <TYPE>      Filter by event type
```

### get

Get event details.

```bash
i-rs-event get <name>
```

### delete

Delete an event.

```bash
i-rs-event delete <name>
```

### stats

View event statistics.

```bash
i-rs-event stats [--year <YEAR>]
```

### example

Show usage examples.

```bash
i-rs-event example
```

### skill

Show AI skill documentation.

```bash
i-rs-event skill [summary|content]
```

## Examples

```bash
# Add a meeting
i-rs-event add "Team Meeting" --date 2024-03-15 --type meeting --location "Conference Room" -p "Alice,Bob" -t work

# Add a gathering
i-rs-event add "Birthday Party" --date 2024-04-20 --type gathering --location "Home" -p "Family,Friends" -t celebration

# List all events
i-rs-event list

# List work events
i-rs-event list --tag work

# View yearly statistics
i-rs-event stats --year 2024
```
