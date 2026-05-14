---
name: "i-rs-bed"
description: "Records bed item replacements. Invoke when user wants to track when they replace mattress, pillows, etc."
---

# i-rs-bed

Bed item replacement tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/beds.json`

## Commands

### add

Record bed item replacement.

```bash
i-rs-bed add <ITEM_TYPE>
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List bed item replacement records.

```bash
i-rs-bed list
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

## Bed Item Types

- `mattress` - Mattress
- `pillow` - Pillow
- `duvet` - Duvet/comforter
- `mattress-protector` - Mattress protector

## Examples

```bash
# Record replacement
i-rs-bed add mattress
i-rs-bed add pillow --tag bedroom

# List records
i-rs-bed list
```