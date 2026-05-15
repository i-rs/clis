---
name: "i-rs-kv"
description: "Stores and retrieves key-value data. Invoke when user wants to store arbitrary key-value data."
---

# i-rs-kv

Key-value storage CLI tool.

## Storage

- Config: `~/.config/i-rs/kv.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a key-value entry.

```bash
i-rs-kv add <KEY> <VALUE> [OPTIONS]
```

Arguments:
- `KEY` - The key name
- `VALUE` - The value to store

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all entries.

```bash
i-rs-kv list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get entry details.

```bash
i-rs-kv get <KEY>
```

### delete

Delete an entry.

```bash
i-rs-kv delete <KEY>
```

### update

Update an entry.

```bash
i-rs-kv update <KEY> [OPTIONS]
```

Options:
- `--value <VALUE>` - Update value
- `-t, --tag <TAG>` - Replace tags
- `-r, --remark <REMARK>` - Replace remarks

### data

Manage data (export, import, clear).

```bash
i-rs-kv data export
i-rs-kv data import [FILE]
i-rs-kv data clear
```

### example

Show usage examples.

```bash
i-rs-kv example
```

### skill

Show skill information.

```bash
i-rs-kv skill [summary|content|raw]
```

## Examples

```bash
# Store values
i-rs-kv add username john [OPTIONS]
i-rs-kv add api-url "https://api.example.com" [OPTIONS]

# List entries
i-rs-kv list [OPTIONS]

# Get value
i-rs-kv get username

# JSON output
i-rs-kv list --json
i-rs-kv get username --json
```
