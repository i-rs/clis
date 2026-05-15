---
name: "i-rs-tick"
description: "Records task duration and time spent. Invoke when user wants to track how long activities or tasks take."
---

# i-rs-tick

Duration tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/tick.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record task duration.

```bash
i-rs-tick add <TASK_NAME> <SECONDS> [OPTIONS]
```

Arguments:
- `TASK_NAME` - Name of the task
- `SECONDS` - Duration in seconds

Options:
- `-s, --started-at <DATETIME>` - Start time (YYYY-MM-DD HH:MM:SS)
- `-d, --description <DESC>` - Description
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List duration records.

```bash
i-rs-tick list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-tick get <ID>
```

### delete

Delete a record.

```bash
i-rs-tick delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-tick data export
i-rs-tick data import [FILE]
i-rs-tick data clear
```

### example

Show usage examples.

```bash
i-rs-tick example
```

### skill

Show skill information.

```bash
i-rs-tick skill [summary|content|raw]
```

## Examples

```bash
# Record task duration
i-rs-tick add coding 3600

# With description and tags
i-rs-tick add meeting 1800 --description "Sprint planning" --tag work

# List records
i-rs-tick list
```
