# i-rs-want Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a wish list item.

```bash
i-rs-want add <NAME> <PRIORITY> [OPTIONS]
```

Arguments:
- `NAME` - Item name
- `PRIORITY` - Priority (low, medium, high)

Options:
- `-u, --url <URL>` - Product URL
- `-p, --price <PRICE>` - Item price
- `-c, --currency <CURRENCY>` - Currency (default: CNY)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List wish list items.

```bash
i-rs-want list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get item details.

```bash
i-rs-want get <NAME>
```

### delete

Delete an item.

```bash
i-rs-want delete <NAME>
```

### update

Update an item.

```bash
i-rs-want update <NAME> [OPTIONS]
```

Options:
- `-p, --price <PRICE>` - Update price
- `-c, --currency <CURRENCY>` - Update currency
- `-u, --url <URL>` - Update URL
- `--priority <PRIORITY>` - Update priority
- `--done` - Mark as done/pending
- `-t, --tag <TAG>` - Update tags
- `-r, --remark <REMARK>` - Update remarks

## Priority Levels

- `low` - Low priority
- `medium` - Medium priority
- `high` - High priority

### data

Manage data (export, import, clear).

```bash
i-rs-want data export
i-rs-want data import [FILE]
i-rs-want data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-want example
```

### skill

Show skill information.

```bash
i-rs-want skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/want.json`
- Linux: `~/.config/i-rs/want.json`
- Windows: `~\AppData\Roaming\i-rs\want.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-want list
```
