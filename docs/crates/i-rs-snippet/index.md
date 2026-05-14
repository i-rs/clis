# i-rs-snippet

Code snippet management CLI tool for storing, organizing, and quickly retrieving code snippets.

## Overview

i-rs-snippet helps you save and organize code snippets with language tags, descriptions, and metadata. Quickly search and copy snippets to clipboard for easy reuse.

## Key Features

- **Multi-language support**: Store snippets from any programming language
- **Tag organization**: Categorize snippets with custom tags
- **Full-text search**: Search by name, language, code content, or tags
- **Clipboard integration**: Copy snippets with one command
- **JSON output**: Machine-readable output for scripting

## Quick Start

```bash
# Add a Rust snippet
i-rs-snippet add hello --language rust --code 'fn main() { println!("Hello!"); }' --tag rust

# List all snippets
i-rs-snippet list

# Search for snippets
i-rs-snippet search print

# Copy to clipboard
i-rs-snippet copy hello
```

## Data Storage

Snippets are stored as JSON in:
- macOS: `~/.config/i-rs/snippets.json`
- Linux: `~/.config/i-rs/snippets.json`
- Windows: `~\AppData\Roaming\i-rs\snippets.json`

Override with `CONFIG_DIR` environment variable.
