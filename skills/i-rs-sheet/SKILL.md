---
name: "i-rs-sheet"
description: "Records sheet changes. Invoke when user wants to track when they change bed sheets."
---

# i-rs-sheet

Sheet change tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/sheets.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record sheet change.

```bash
i-rs-sheet add <SHEET_TYPE> [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List sheet change records.

```bash
i-rs-sheet list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-sheet get <ID>
```

### delete

Delete a record.

```bash
i-rs-sheet delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-sheet data export
i-rs-sheet data import [FILE]
i-rs-sheet data clear
```

### example

Show usage examples.

```bash
i-rs-sheet example
```

### skill

Show skill information.

```bash
i-rs-sheet skill [summary|content|raw]
```

## Sheet Types

- `bedsheet` - Bottom or top sheet
- `pillowcase` - Pillow cover
- `duvet-cover` - Comforter/duvet cover
- `mattress-protector` - Mattress protector

## Examples

```bash
# Record change
i-rs-sheet add bedsheet
i-rs-sheet add pillowcase --tag bedroom

# List records
i-rs-sheet list [OPTIONS]
```
