---
name: "i-rs-cal"
description: "Records calorie intake estimates. Invoke when user wants to track food calories."
---

# i-rs-cal

Calorie tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/cal.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record calorie intake.

```bash
i-rs-cal add <FOOD_NAME> <CALORIES> [OPTIONS]
```

Options:
- `-d, --date <DATE>` - Date in YYYY-MM-DD format
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List calorie records.

```bash
i-rs-cal list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-cal get <ID>
```

### delete

Delete a record.

```bash
i-rs-cal delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-cal data export
i-rs-cal data import [FILE]
i-rs-cal data clear
```

### example

Show usage examples.

```bash
i-rs-cal example
```

### skill

Show skill information.

```bash
i-rs-cal skill [summary|content|raw]
```

## Examples

```bash
# Record calories
i-rs-cal add "Apple" 95 [OPTIONS]
i-rs-cal add "Pizza" 285 --tag lunch [OPTIONS]

# List records
i-rs-cal list [OPTIONS]
```