---
name: "i-rs-mood"
description: "Tracks mood records (add/list/update/delete). Invoke when user needs to record daily mood, view mood history, or display mood statistics and calendar."
---

# i-rs-mood

Mood tracking CLI tool for recording and visualizing daily mood.

## Storage

- Config: `~/.config/i-rs/moods.json`

## Global Flags

- `--json` — Output in JSON format
## Mood Levels

| Input | Emoji | Label |
|-------|-------|-------|
| 5, great, 😊 | 😊 | Great |
| 4, good, 🙂 | 🙂 | Good |
| 3, okay/ok, 😐 | 😐 | Okay |
| 2, bad, 😔 | 😔 | Bad |
| 1, terrible, 😢 | 😢 | Terrible |

## Commands

### add

Add a mood record.

```bash
i-rs-mood add <DATE> <MOOD>
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-c, --content <CONTENT>` - Content/notes (can be repeated)

### list

List mood records.

```bash
i-rs-mood list
```

Options:
- `-d, --days <DAYS>` - Show records from last N days
- `-c, --calendar` - Show mood calendar

### update

Update a mood record.

```bash
i-rs-mood update <DATE>
```

Options:
- `-m, --mood <MOOD>` - New mood level
- `-t, --tag <TAG>` - New tags
- `-c, --content <CONTENT>` - New content

### delete

Delete a mood record.

```bash
i-rs-mood delete <DATE>
```

### data

Manage data (export, import, clear).

```bash
i-rs-mood data export
i-rs-mood data import [FILE]
i-rs-mood data clear
```

### example

Show usage examples.

```bash
i-rs-mood example
```

### skill

Show skill information.

```bash
i-rs-mood skill [summary|content|raw]
```

## Examples

```bash
# Record today's mood
i-rs-mood add 2025-01-15 good

# Record with emoji
i-rs-mood add 2025-01-16 😊 --tag weekend --content "Great day"

# List all records
i-rs-mood list

# List last 7 days with calendar
i-rs-mood list --days 7 --calendar

# Update mood
i-rs-mood update 2025-01-15 --mood okay

# Delete record
i-rs-mood delete 2025-01-15
```
