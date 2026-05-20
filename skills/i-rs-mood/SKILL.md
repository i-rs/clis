---
name: "i-rs-mood"
description: "Tracks mood records (add/list/update/delete). Invoke when user needs to record daily mood, view mood history, or display mood statistics and calendar."
---

# i-rs-mood

Mood tracking CLI tool for recording and visualizing daily mood.

## Storage

- Config: `~/.config/i-rs/mood.json`

## Global Flags

- `--json` — Output in JSON format
## Mood Levels

| Input | Emoji | Label |
|-------|-------|-------|
| 7, amazing, 🤩 | 🤩 | Amazing |
| 6, great, 😊 | 😊 | Great |
| 5, good, 🙂 | 🙂 | Good |
| 4, okay/ok, 😐 | 😐 | Okay |
| 3, poor, 😕 | 😕 | Poor |
| 2, bad, 😔 | 😔 | Bad |
| 1, terrible, 😢 | 😢 | Terrible |

## Commands

### add

Add a mood record.

```bash
i-rs-mood add <MOOD> [OPTIONS]
```

Arguments:
- `MOOD` - Mood level (see Mood Levels below)

Options:
- `-D, --date <DATE>` - Date in YYYY-MM-DD format (default: today)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List mood records.

```bash
i-rs-mood list [OPTIONS]
```

Options:
- `-d, --days <DAYS>` - Show records from last N days
- `-c, --calendar` - Show mood calendar

### get

Get a mood record by id.

```bash
i-rs-mood get <ID>
```

### update

Update a mood record by id.

```bash
i-rs-mood update <ID> [OPTIONS]
```

Options:
- `-D, --date <DATE>` - New date
- `-m, --mood <MOOD>` - New mood level
- `-t, --tag <TAG>` - New tags
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### delete

Delete a mood record by id.

```bash
i-rs-mood delete <ID>
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
i-rs-mood add good [OPTIONS]

# Record mood for a specific date
i-rs-mood add 😊 --date 2025-01-16 --tag weekend --remark "Great day"

# List all records
i-rs-mood list [OPTIONS]

# List last 7 days with calendar
i-rs-mood list --days 7 --calendar

# Get record by id
i-rs-mood get abc12345

# Update mood
i-rs-mood update abc12345 --mood okay

# Delete record
i-rs-mood delete abc12345
```
