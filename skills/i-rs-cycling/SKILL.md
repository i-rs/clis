---
name: "i-rs-cycling"
description: "Tracks cycling activities (add/list/get/update/delete). Invoke when user needs to record cycling workouts, view cycling history, or display statistics."
---

# i-rs-cycling

Cycling record tracking CLI tool for recording and managing cycling activities.

## Storage

- Config: `~/.config/i-rs/cycling.json`

## Commands

### add
Add a cycling record.
```bash
i-rs-cycling add <DATE> <DISTANCE> <DURATION> [--elevation] [--route] [--tag] [--remark]
```

### list
List cycling records.
```bash
i-rs-cycling list [--tag TAG]
```

### get
View record details.
```bash
i-rs-cycling get <ID|DATE>
```

### update
Update a cycling record.
```bash
i-rs-cycling update <ID|DATE> [--distance] [--duration] [--elevation] [--route] [--add-tag] [--remove-tag] [--add-remark]
```

### delete
Delete a cycling record.
```bash
i-rs-cycling delete <ID|DATE>
```

### stats
View cumulative statistics.
```bash
i-rs-cycling stats
```

## Examples

```bash
# Add a simple record
i-rs-cycling add 2025-06-14 25.5 60

# Add with elevation
i-rs-cycling add 2025-06-15 30.2 75 --elevation 450

# Add with tags and route
i-rs-cycling add 2025-06-16 15.0 30 --route "Morning ride" --tag morning

# List records
i-rs-cycling list

# Filter by tag
i-rs-cycling list --tag mountain

# View statistics
i-rs-cycling stats

# Get record details
i-rs-cycling get <uuid>

# Update record
i-rs-cycling update <uuid> --distance 26.0

# Delete record
i-rs-cycling delete <uuid>
```
