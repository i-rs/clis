---
name: "i-rs-want"
description: "Tracks wish list items. Invoke when user wants to manage things they want to buy or get."
---

# i-rs-want

Wish list CLI tool.

## Storage

- Config: `~/.config/i-rs/want.json`

## Commands

### add

Add a wish list item.

```bash
i-rs-want add <NAME> [OPTIONS]
```

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

## Examples

```bash
# Add wish list items
i-rs-want add "New Headphones" --price 299.99 --priority high
i-rs-want add "Book: Rust Programming" --price 49.99 --priority medium
i-rs-want add "Keyboard" --url "https://example.com/keyboard" --priority high

# List items
i-rs-want list

# Mark as done
i-rs-want update "New Headphones" --done
```