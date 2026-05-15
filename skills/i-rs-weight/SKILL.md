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
i-rs-weight add <DATE> <WEIGHT> [OPTIONS]
```

Options:
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

### update

Update a weight record.

```bash
i-rs-weight update <DATE> [OPTIONS]
```

Options:
- `-w, --weight <WEIGHT>` - New weight value
- `-r, --remark <REMARK>` - New remarks

### delete

Delete a weight record.

```bash
i-rs-weight delete <DATE>
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
i-rs-weight add 2025-01-15 70.5

# Add with remarks
i-rs-weight add 2025-01-16 70.3 --remark "After workout"

# List all records
i-rs-weight list

# List last 30 days with chart and stats
i-rs-weight list --days 30 --chart --stats

# Show only chart
i-rs-weight list --chart

# Show only stats
i-rs-weight list --stats

# Update a record
i-rs-weight update 2025-01-15 --weight 70.0

# Delete a record
i-rs-weight delete 2025-01-15
```
