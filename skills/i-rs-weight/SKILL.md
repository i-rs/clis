---
name: "i-rs-weight"
description: "Manages weight records (add/list/update/delete). Invoke when user needs to track weight, view weight history, show trends, or display statistics."
---

# i-rs-weight

Weight tracking CLI tool for managing weight records with trend visualization.

## Storage

- Config: `~/.config/i-rs/weights.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add a weight record.

```bash
i-rs-weight add <WEIGHT> [OPTIONS]
```

Args:
- `WEIGHT` — Weight value (kg)

Options:
- `-d, --date <DATE>` - Date YYYY-MM-DD (default: today)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List weight records.

```bash
i-rs-weight list [OPTIONS]
```

Options:
- `-d, --days <DAYS>` - Show records from last N days
- `-c, --chart` - Show ASCII trend chart
- `-s, --stats` - Show statistics (min/max/avg/change)

### get

Get a weight record by ID.

```bash
i-rs-weight get <ID>
```

### update

Update a weight record.

```bash
i-rs-weight update <ID> [OPTIONS]
```

Options:
- `-w, --weight <WEIGHT>` - New weight value
- `-t, --tag <TAG>` - New tags (replaces all)
- `-r, --remark <REMARK>` - New remarks (replaces all)

### delete

Delete a weight record.

```bash
i-rs-weight delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-weight data export
i-rs-weight data import [FILE]
i-rs-weight data clear
```

### example

Show usage examples.

```bash
i-rs-weight example
```

### skill

Show skill information.

```bash
i-rs-weight skill [summary|content|raw]
```

## Examples

```bash
# Add a weight record
i-rs-weight add 70.5

# Add with date and remarks
i-rs-weight add 70.3 --date 2025-01-16 --remark "After workout"

# List all records
i-rs-weight list

# List last 30 days with chart and stats
i-rs-weight list --days 30 --chart --stats

# Show only chart
i-rs-weight list --chart

# Show only stats
i-rs-weight list --stats

# Get a record
i-rs-weight get abc12345

# Update a record
i-rs-weight update abc12345 --weight 70.0

# Delete a record
i-rs-weight delete abc12345
```
