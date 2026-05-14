# i-rs-grocery Usage

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

## Data Storage

- macOS: `~/Library/Application Support/i-rs/grocery.json`
- Linux: `~/.config/i-rs/grocery.json`
- Windows: `~\AppData\Roaming\i-rs\grocery.json`