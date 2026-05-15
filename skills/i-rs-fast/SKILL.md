---
name: "i-rs-fast"
description: "Records fasting sessions. Invoke when user wants to track fasting."
---

# i-rs-fast

Fasting tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/fast.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Start a fast.

```bash
i-rs-fast add <TARGET_HOURS>
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List fasting records.

```bash
i-rs-fast list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-fast get <ID>
```

### delete

Delete a record.

```bash
i-rs-fast delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-fast data export
i-rs-fast data import [FILE]
i-rs-fast data clear
```

### example

Show usage examples.

```bash
i-rs-fast example
```

### skill

Show skill information.

```bash
i-rs-fast skill [summary|content|raw]
```

## Examples

```bash
# Start fasting
i-rs-fast add 16
i-rs-fast add 24 --tag omad

# List records
i-rs-fast list
```