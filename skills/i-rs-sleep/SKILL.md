---
name: "i-rs-sleep"
description: "Tracks sleep patterns with bedtime, wake time, and quality ratings. Invoke when user wants to record or review sleep data."
---

# i-rs-sleep

Sleep tracking CLI tool for recording bedtime, wake time, and sleep quality.

## Storage

- Config: `~/.config/i-rs/sleep.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record a sleep session.

```bash
i-rs-sleep add <BEDTIME> <WAKE_TIME> <QUALITY> [OPTIONS]
```

Arguments:
- `BEDTIME` - Bedtime in HH:MM format
- `WAKE_TIME` - Wake time in HH:MM format  
- `QUALITY` - Sleep quality (1-5)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all sleep records.

```bash
i-rs-sleep list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get sleep record details.

```bash
i-rs-sleep get <ID>
```

### stats

Show sleep statistics.

```bash
i-rs-sleep stats
```

### update

Update a sleep record.

```bash
i-rs-sleep update <ID> [OPTIONS]
```

### delete

Delete a sleep record.

```bash
i-rs-sleep delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-sleep data export
i-rs-sleep data import [FILE]
i-rs-sleep data clear
```

### example

Show usage examples.

```bash
i-rs-sleep example
```

### skill

Show skill information.

```bash
i-rs-sleep skill [summary|content|raw]
```

## Examples

```bash
# Record sleep
i-rs-sleep add 22:30 06:45 4 --tag workday [OPTIONS]

# View statistics
i-rs-sleep stats

# List records by tag
i-rs-sleep list --tag weekend
```

## Quality Scale

| Rating | Emoji | Description |
|--------|-------|-------------|
| 1 | 😴 | Awful |
| 2 | 😪 | Poor |
| 3 | 😌 | Fair |
| 4 | 😊 | Good |
| 5 | 😁 | Excellent |