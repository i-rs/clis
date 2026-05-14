---
name: "i-rs-kv"
description: "Stores and retrieves key-value data. Invoke when user wants to store arbitrary key-value data."
---

# i-rs-kv

Key-value storage CLI tool.

## Storage

- Config: `~/.config/i-rs/kv.json`

## Commands

### add

Add a key-value entry.

```bash
i-rs-kv add <KEY> --value <VALUE> [OPTIONS]
```

Options:
- `--value <VALUE>` - The value to store
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all entries.

```bash
i-rs-kv list
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
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

## Examples

```bash
# Store values
i-rs-kv add "username" --value "john"
i-rs-kv add "api-url" --value "https://api.example.com"

# List entries
i-rs-kv list

# Get value
i-rs-kv get username
```