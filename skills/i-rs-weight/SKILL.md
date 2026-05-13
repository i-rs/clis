---
name: "i-rs-weight"
description: "Manages weight records (add/list/update/delete). Invoke when user needs to track weight, view weight history, show trends, or display statistics."
---

# i-rs-weight

Weight tracking CLI tool for managing weight records with trend visualization.

## Storage

- Config: `~/.config/i-rs/weights.json`

## Commands

### add

Add a weight record.

```bash
i-rs-weight add <DATE> <WEIGHT>
```

Options:
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List weight records.

```bash
i-rs-weight list
```

Options:
- `-d, --days <DAYS>` - Show records from last N days
- `-c, --chart` - Show ASCII trend chart
- `-s, --stats` - Show statistics (min/max/avg/change)

### update

Update a weight record.

```bash
i-rs-weight update <DATE>
```

Options:
- `-w, --weight <WEIGHT>` - New weight value
- `-r, --remark <REMARK>` - New remarks

### delete

Delete a weight record.

```bash
i-rs-weight delete <DATE>
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
