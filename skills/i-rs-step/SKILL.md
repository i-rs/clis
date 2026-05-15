---
name: "i-rs-step"
description: "Records daily step count. Invoke when user wants to track walking or running steps."
---

# i-rs-step

Step counting CLI tool.

## Storage

- Config: `~/.config/i-rs/step.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record daily steps.

```bash
i-rs-step add <STEPS> <DATE> [OPTIONS]
```

Arguments:
- `STEPS` - Number of steps
- `DATE` - Date (YYYY-MM-DD)

Options:
- `-d, --distance <KM>` - Distance in kilometers
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List step records.

```bash
i-rs-step list
```

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
- `-s, --steps <STEPS>` - Update steps
- `-d, --distance <KM>` - Update distance
- `-t, --tag <TAG>` - Update tags
- `-r, --remark <REMARK>` - Update remarks

### data

Manage data (export, import, clear).

```bash
i-rs-step data export
i-rs-step data import [FILE]
i-rs-step data clear
```

### example

Show usage examples.

```bash
i-rs-step example
```

### skill

Show skill information.

```bash
i-rs-step skill [summary|content|raw]
```

## Examples

```bash
# Record steps
i-rs-step add 10000 2024-01-15
i-rs-step add 8000 2024-01-16 --distance 6.4

# List records
i-rs-step list

# Get details
i-rs-step get 2024-01-15
```
