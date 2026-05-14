---
name: "i-rs-quote"
description: "Manages quotes (add/list/get/delete/random). Invoke when user needs to store, retrieve, or organize inspirational quotes."
---

# i-rs-quote

语录收藏 CLI 工具 - 收集、整理和展示名人名言、书籍摘录等。

## Storage

- Config: `~/.config/i-rs/quotes.json`

## Commands

### add
Add a new quote.

```bash
i-rs-quote add --content "Quote text" [--author "Author"] [--source "Source"] [--tag TAG] [--remark "Note"]
```

Options:
- `--content, -c`: Quote content (required)
- `--author, -a`: Quote author
- `--source, -s`: Quote source (book, speech, etc.)
- `--tag, -g`: Tags (repeatable)
- `--remark, -r`: Personal remarks (repeatable)

### list
List all quotes or filter by tag/author.

```bash
i-rs-quote list [--tag TAG] [--author AUTHOR]
```

Options:
- `--tag, -t`: Filter by tag
- `--author, -a`: Filter by author (fuzzy match)

### get
Get quote details.

```bash
i-rs-quote get <ID>
```

### delete
Delete a quote.

```bash
i-rs-quote delete <ID>
```

### random
Display a random quote.

```bash
i-rs-quote random
```

### example
Show usage examples.

```bash
i-rs-quote example
```

### skill
View AI skill documentation.

```bash
i-rs-quote skill          # Show raw skill document
i-rs-quote skill summary  # Show summary
i-rs-quote skill content  # Show content
```

## Examples

```bash
# Add a simple quote
i-rs-quote add --content "Stay hungry, stay foolish."

# Add with author and tags
i-rs-quote add --content "The only way to do great work is to love what you do." --author "Steve Jobs" --source "Stanford Speech" --tag inspiration --tag career

# List all quotes
i-rs-quote list

# Filter by tag
i-rs-quote list --tag inspiration

# Search by author
i-rs-quote list --author "Steve"

# Get quote details
i-rs-quote get <uuid>

# Random quote
i-rs-quote random

# Delete a quote
i-rs-quote delete <uuid>

# JSON output
i-rs-quote list --json
i-rs-quote get <uuid> --json
```
