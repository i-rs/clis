---
name: "i-rs-towel"
description: "Records towel replacements. Invoke when user wants to track when they replace towels."
---

# i-rs-towel

Towel replacement tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/towels.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Record towel replacement.

```bash
i-rs-towel add <TOWEL_TYPE>
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List towel replacement records.

```bash
i-rs-towel list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-towel get <ID>
```

### delete

Delete a record.

```bash
i-rs-towel delete <ID>
```

## Towel Types

- `bath` - Bath towel
- `face` - Face towel
- `hand` - Hand towel
- `beach` - Beach towel
- `sports` - Sports towel

### data

Manage data (export, import, clear).

```bash
i-rs-towel data export
i-rs-towel data import [FILE]
i-rs-towel data clear
```

### example

Show usage examples.

```bash
i-rs-towel example
```

### skill

Show skill information.

```bash
i-rs-towel skill [summary|content|raw]
```

## Examples

```bash
# Record replacement
i-rs-towel add bath
i-rs-towel add face --tag bedroom

# List records
i-rs-towel list
```