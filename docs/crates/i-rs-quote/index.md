# i-rs-quote

Quote collection CLI tool for collecting, organizing, and displaying quotes and book excerpts.

## Introduction

i-rs-quote is a CLI tool for managing quotes. It supports adding quotes with author and source information, organizing them with tags, searching by author, and randomly displaying quotes.

## Quick Start

```bash
# Add a quote
i-rs-quote add --content "The only way to do great work is to love what you do." --author "Steve Jobs"

# List all quotes
i-rs-quote list

# Random quote
i-rs-quote random

# View details
i-rs-quote get <uuid>
```

## Data Storage

- macOS: `~/.config/i-rs/quotes.json`
- Linux: `~/.config/i-rs/quotes.json`
- Windows: `~\AppData\Roaming\i-rs\quotes.json`

## License

MIT OR Apache-2.0
