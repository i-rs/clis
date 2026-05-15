# i-rs-grocery Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add an item to the grocery list.

```bash
i-rs-grocery add <NAME> [QUANTITY] [UNIT] [OPTIONS]
```

Arguments:
- `NAME` - Item name (required)
- `QUANTITY` - Quantity (default: 1)
- `UNIT` - Unit (default: item)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List grocery items.

```bash
i-rs-grocery list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag
- `--purchased` - Show only purchased items
- `-n, --needed` - Show only needed items

### purchase

Toggle purchase status of an item.

```bash
i-rs-grocery purchase <NAME>
```

Arguments:
- `NAME` - Item name (required)

### get

Get item details.

```bash
i-rs-grocery get <NAME>
```

Arguments:
- `NAME` - Item name (required)

### update

Update an item.

```bash
i-rs-grocery update <NAME> [OPTIONS]
```

Options:
- `-q, --quantity <NUM>` - Update quantity
- `-u, --unit <UNIT>` - Update unit
- `-t, --tag <TAG>` - Update tags
- `-r, --remark <REMARK>` - Update remarks

### clear

Clear all purchased items.

```bash
i-rs-grocery clear
```

### delete

Delete an item.

```bash
i-rs-grocery delete <NAME>
```

Arguments:
- `NAME` - Item name (required)

## Common Units

| Unit | Description |
|------|-------------|
| item | Single item |
| kg | Kilogram |
| g | Gram |
| liter | Liter |
| bottle | Bottle |
| pack | Pack |
| dozen | Dozen (12) |
| loaf | Loaf of bread |
| box | Box |

### data

Manage data (export, import, clear).

```bash
i-rs-grocery data export
i-rs-grocery data import [FILE]
i-rs-grocery data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-grocery example
```
### skill

Show skill information.

```bash
i-rs-grocery skill [summary|content|raw]
```

## Data Storage

- macOS: `~/Library/Application Support/i-rs/grocery.json`
- Linux: `~/.config/i-rs/grocery.json`
- Windows: `~\AppData\Roaming\i-rs\grocery.json`