# i-rs-bestby Usage

## Commands

### add

Add an item with purchase date and replacement cycle.

```bash
i-rs-bestby add <NAME> --purchase-date <DATE> [OPTIONS]
```

Arguments:
- `NAME` - Item name

Options:
- `--purchase-date <DATE>` - Purchase date (YYYY-MM-DD)
- `--cycle-days <DAYS>` - Replacement cycle in days
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all items.

```bash
i-rs-bestby list
```

### get

Get item details.

```bash
i-rs-bestby get <NAME>
```

### delete

Delete an item.

```bash
i-rs-bestby delete <NAME>
```

### update

Update an item.

```bash
i-rs-bestby update <NAME> [OPTIONS]
```

Options:
- `--purchase-date <DATE>` - Update purchase date
- `--cycle-days <DAYS>` - Update replacement cycle
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

## Data Storage

- macOS: `~/.config/i-rs/bestby.json`
- Linux: `~/.config/i-rs/bestby.json`
- Windows: `~\AppData\Roaming\i-rs\bestby.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-bestby list
```