# i-rs-want Usage

## Commands

### add

Add a wish list item.

```bash
i-rs-want add <NAME> [OPTIONS]
```

Arguments:
- `NAME` - Item name

Options:
- `--price <PRICE>` - Item price
- `--currency <CURRENCY>` - Currency (default: CNY)
- `--url <URL>` - Product URL
- `--priority <PRIORITY>` - Priority (low, medium, high)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List wish list items.

```bash
i-rs-want list
```

Options:
- `--done` - Show completed items
- `--pending` - Show pending items (default)

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
- `--price <PRICE>` - Update price
- `--url <URL>` - Update URL
- `--priority <PRIORITY>` - Update priority
- `--done` - Mark as done
- `--undone` - Mark as pending
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

## Priority Levels

- `low` - Low priority
- `medium` - Medium priority
- `high` - High priority

## Data Storage

- macOS: `~/.config/i-rs/want.json`
- Linux: `~/.config/i-rs/want.json`
- Windows: `~\AppData\Roaming\i-rs\want.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-want list
```