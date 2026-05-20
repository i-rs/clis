# i-rs-kv Usage

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
i-rs-kv list
```

Options:
- `-t, --tag <TAG>` - Filter by tag
- `-p, --pattern <PATTERN>` - Filter by key or value pattern

### get

Get entry details.

```bash
i-rs-kv get <KEY>
```

### search

Search entries by key, value, tags, or remarks.

```bash
i-rs-kv search <QUERY>
```

Arguments:
- `QUERY` - Search term (case-insensitive)

### stats

Show KV store statistics.

```bash
i-rs-kv stats
```

### copy

Copy an entry to a new key.

```bash
i-rs-kv copy <SRC_KEY> <DST_KEY>
```

### rename

Rename an entry key.

```bash
i-rs-kv rename <OLD_KEY> <NEW_KEY>
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

Manage stored data.

```bash
i-rs-kv data export
i-rs-kv data import [FILE]
i-rs-kv data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

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

## Data Storage

- macOS: `~/.config/i-rs/kv.json`
- Linux: `~/.config/i-rs/kv.json`
- Windows: `~\AppData\Roaming\i-rs\kv.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-kv list
```
