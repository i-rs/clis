---
name: "i-rs-bed"
description: "Records bed item replacements. Invoke when user wants to track when they replace mattress, pillows, etc."
---

# i-rs-bed

Bed item replacement tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/beds.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record bed item replacement.

```bash
i-rs-bed add <ITEM_TYPE> [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List bed item replacement records.

```bash
i-rs-bed list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-bed get <ID>
```

### delete

Delete a record.

```bash
i-rs-bed delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-bed data export
i-rs-bed data import [FILE]
i-rs-bed data clear
```

### example

Show usage examples.

```bash
i-rs-bed example
```

### skill

Show skill information.

```bash
i-rs-bed skill [summary|content|raw]
```

## Bed Item Types

- `mattress` - Mattress
- `pillow` - Pillow
- `duvet` - Duvet/comforter
- `mattress-protector` - Mattress protector

## Examples

```bash
# Record replacement
i-rs-bed add mattress [OPTIONS]
i-rs-bed add pillow --tag bedroom [OPTIONS]

# List records
i-rs-bed list [OPTIONS]
```
