---
name: "i-rs-step"
description: "Records daily step count. Invoke when user wants to track walking or running steps."
---

# i-rs-step

Step counting CLI tool.

## Storage

- Config: `~/.config/i-rs/step.json`

## Commands

### add

Record daily steps.

```bash
i-rs-step add <STEPS> [OPTIONS]
```

Arguments:
- `STEPS` - Number of steps

Options:
- `--distance <KM>` - Distance in kilometers
- `--date <DATE>` - Date (YYYY-MM-DD, default: today)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List step records.

```bash
i-rs-step list
```

Options:
- `--date <DATE>` - Filter by date

### get

Get record details.

```bash
i-rs-step get <DATE>
```

### delete

Delete a record.

```bash
i-rs-step delete <DATE>
```

### update

Update a record.

```bash
i-rs-step update <DATE> [OPTIONS]
```

Options:
- `--steps <STEPS>` - Update steps
- `--distance <KM>` - Update distance
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

## Examples

```bash
# Record steps
i-rs-step add 10000
i-rs-step add 8000 --distance 6.4

# List records
i-rs-step list

# Get details
i-rs-step get 2024-01-15
```