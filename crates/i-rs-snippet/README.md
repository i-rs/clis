# i-rs-snippet

Code snippet management CLI tool for storing, organizing, and quickly retrieving code snippets.

## Features

- Store code snippets with language, tags, and descriptions
- Search snippets by name, language, code content, or tags
- Copy snippets to clipboard with one command
- Support multiple programming languages
- Tag support for organization
- JSON output for scripting

## Install

```bash
npm install -g @i-rs/i-rs-snippet
# or
brew install i-rs/homebrew-tap/i-rs-snippet
```

## Quick Start

```bash
# Add a code snippet
i-rs-snippet add hello --language rust --code 'fn main() { println!("Hello!"); }' --tag rust

# List all snippets
i-rs-snippet list

# Search snippets
i-rs-snippet search hello

# Get snippet details
i-rs-snippet get hello

# Copy to clipboard
i-rs-snippet copy hello
```

## Commands

| Command | Description |
|---------|-------------|
| `add` | Add a new code snippet |
| `list` | List all snippets |
| `search` | Search snippets by query |
| `get` | Get snippet details |
| `copy` | Copy snippet to clipboard |
| `update` | Update a snippet |
| `delete` | Delete a snippet |
| `example` | Show usage examples |
| `skill` | View AI skill documentation |

## Data Storage

- macOS: `~/.config/i-rs/snippets.json`
- Linux: `~/.config/i-rs/snippets.json`
- Windows: `~\AppData\Roaming\i-rs\snippets.json`

## License

MIT OR Apache-2.0
