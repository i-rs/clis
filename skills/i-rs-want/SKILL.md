---
name: "i-rs-want"
description: "Tracks wish list items. Invoke when user wants to manage things they want to buy or get."
---

# i-rs-want

Wish list CLI tool.

## Storage

- Config: `~/.config/i-rs/want.json`

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

### data

Manage data (export, import, clear).

```bash
i-rs-want data export
i-rs-want data import [FILE]
i-rs-want data clear
```

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

## Examples

```bash
# Add wish list items
i-rs-want add "New Headphones" high --price 299.99
i-rs-want add "Book: Rust Programming" medium --price 49.99
i-rs-want add "Keyboard" high --url "https://example.com/keyboard"

# List items
i-rs-want list

# Mark as done
i-rs-want update "New Headphones" --done
```
