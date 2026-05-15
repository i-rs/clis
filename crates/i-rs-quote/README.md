# i-rs-quote

Quote collection CLI tool for collecting, organizing, and displaying quotes and book excerpts.

## Features

- Quote content management (content, author, source)
- Tag classification system
- Search by author
- Random quote display
- Personal notes support
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-quote
# or
brew install i-rs/homebrew-tap/i-rs-quote
```

## Quick Start

```bash
# Add a quote
i-rs-quote add --content "The only way to do great work is to love what you do." --author "Steve Jobs"

# Search by author
i-rs-quote list --author "Steve"

# List all quotes
i-rs-quote list

# Random quote
i-rs-quote random

# View details
i-rs-quote get <uuid>

# Delete a quote
i-rs-quote delete <uuid>
```

## Commands

### add - Add a quote

```bash
i-rs-quote add --content "Quote text" [--author "Author"] [--source "Source"] [--tag TAG] [--remark "Note"]
```

### list - List quotes

```bash
i-rs-quote list [--tag TAG] [--author AUTHOR]
```

### get - View details

```bash
i-rs-quote get <ID>
```

### random - Random display

```bash
i-rs-quote random
```

### delete - Delete quote

```bash
i-rs-quote delete <ID>
```

## Data Storage

- macOS: `~/.config/i-rs/quotes.json`
- Linux: `~/.config/i-rs/quotes.json`
- Windows: `~\AppData\Roaming\i-rs\quotes.json`

## JSON Output

All commands support `--json` global flag:

```bash
i-rs-quote list --json
i-rs-quote get <ID> --json
```

## License

MIT OR Apache-2.0
