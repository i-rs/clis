---
name: "i-rs-sit"
description: "Records sitting duration. Invoke when user wants to track how long they've been sitting."
---

# i-rs-sit

Sitting duration tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/sit.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record sitting duration.

```bash
i-rs-sit add <DURATION_MINUTES> [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List sitting records.

```bash
i-rs-sit list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-sit get <ID>
```

### delete

Delete a record.

```bash
i-rs-sit delete <ID>
```

### update

Update a sitting record.

```bash
i-rs-sit update <ID> [OPTIONS]
```

Options:
- `--duration-minutes <MIN>` - Duration in minutes
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### data

Manage data (export, import, clear).

```bash
i-rs-sit data export
i-rs-sit data import [FILE]
i-rs-sit data clear
```

### example

Show usage examples.

```bash
i-rs-sit example
```

### skill

Show skill information.

```bash
i-rs-sit skill [summary|content|raw]
```

## Examples

```bash
# Record sitting
i-rs-sit add 60 [OPTIONS]
i-rs-sit add 120 --tag work [OPTIONS]

# List records
i-rs-sit list [OPTIONS]
```