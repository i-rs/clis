---
name: "i-rs-water"
description: "Records water intake. Invoke when user wants to track how much water they drink."
---

# i-rs-water

Water intake tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/water.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record water intake.

```bash
i-rs-water add <AMOUNT_ML> [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List water records.

```bash
i-rs-water list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-water get <ID>
```

### delete

Delete a record.

```bash
i-rs-water delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-water data export
i-rs-water data import [FILE]
i-rs-water data clear
```

### example

Show usage examples.

```bash
i-rs-water example
```

### skill

Show skill information.

```bash
i-rs-water skill [summary|content|raw]
```

## Examples

```bash
# Record water intake
i-rs-water add 250
i-rs-water add 500 --tag morning

# List records
i-rs-water list

# Get details
i-rs-water get abc12345
```