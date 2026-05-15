---
name: "i-rs-aqua"
description: "Records aquarium water changes. Invoke when user wants to track when they change fish tank water."
---

# i-rs-aqua

Aquarium water change tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/aqua.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Record water change.

```bash
i-rs-aqua add
```

Options:
- `-t, --tank-size <SIZE>` - Tank size in liters
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List water change records.

```bash
i-rs-aqua list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-aqua get <ID>
```

### delete

Delete a record.

```bash
i-rs-aqua delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-aqua data export
i-rs-aqua data import [FILE]
i-rs-aqua data clear
```

### example

Show usage examples.

```bash
i-rs-aqua example
```

### skill

Show skill information.

```bash
i-rs-aqua skill [summary|content|raw]
```

## Examples

```bash
# Record water change
i-rs-aqua add
i-rs-aqua add --tank-size 100

# List records
i-rs-aqua list
```