# i-rs-kv Usage

## Commands

### add

Add a key-value entry.

```bash
i-rs-kv add <KEY> --value <VALUE> [OPTIONS]
```

Arguments:
- `KEY` - The key name

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

## Data Storage

- macOS: `~/.config/i-rs/kv.json`
- Linux: `~/.config/i-rs/kv.json`
- Windows: `~\AppData\Roaming\i-rs\kv.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-kv list
```