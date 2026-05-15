---
name: "i-rs-ac"
description: "Records AC cleaning. Invoke when user wants to track when they clean air conditioners."
---

# i-rs-ac

AC cleaning tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/ac.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record AC cleaning.

```bash
i-rs-ac add <LOCATION> [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List AC cleaning records.

```bash
i-rs-ac list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-ac get <ID>
```

### delete

Delete a record.

```bash
i-rs-ac delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-ac data export
i-rs-ac data import [FILE]
i-rs-ac data clear
```

### example

Show usage examples.

```bash
i-rs-ac example
```

### skill

Show skill information.

```bash
i-rs-ac skill [summary|content|raw]
```

## Examples

```bash
# Record cleaning
i-rs-ac add "Living Room" [OPTIONS]
i-rs-ac add "Bedroom" --tag summer [OPTIONS]

# List records
i-rs-ac list [OPTIONS]
```