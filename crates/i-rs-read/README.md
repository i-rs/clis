# i-rs-read

Reading progress tracking CLI tool to help you track and manage books you are reading.

## Features

- Add and manage book info (title, author, total pages)
- Track reading progress (current page, completion percentage)
- Multiple reading statuses (Reading, Completed, Paused, Dropped, ToRead)
- Book rating and review
- Tag-based organization
- Reading statistics (total books, pages, completed, average rating)
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-read
# or
brew install i-rs/homebrew-tap/i-rs-read
```

## Quick Start

```bash
# Add a new book
i-rs-read add "The Rust Programming Language" "Steve Klabnik" 500

# Update reading progress
i-rs-read update "The Rust Programming Language" --current-page 250

# Mark as completed with rating
i-rs-read update "The Rust Programming Language" --status completed --rating 5

# List all books
i-rs-read list

# View reading statistics
i-rs-read stats
```

## Commands

| Command | Description |
|---------|-------------|
| `add` | Add a new book |
| `list` | List all books |
| `get` | Get book details |
| `update` | Update book information |
| `delete` | Delete a book |
| `stats` | Show reading statistics |
| `example` | Show usage examples |
| `skill` | Show AI skill documentation |

### add - Add a book

```bash
i-rs-read add <NAME> <AUTHOR> <TOTAL_PAGES> [OPTIONS]

Options:
  --tags <TAGS>       Add tags (repeatable)
  --remark <REMARK>   Add remarks (repeatable)
```

### list - List books

```bash
i-rs-read list [OPTIONS]

Options:
  --tag <TAG>         Filter by tag
  --status <STATUS>   Filter by status (reading, completed, paused, dropped, to_read)
```

### update - Update book

```bash
i-rs-read update <NAME> [OPTIONS]

Options:
  --current-page <PAGE>   Update current page
  --status <STATUS>       Update reading status
  --rating <RATING>       Add rating (0-5)
  --review <REVIEW>       Add review
  --tags <TAGS>           Update tags (comma separated)
  --add-remark <REMARK>   Add remark
  --remove-remark <INDEX> Remove remark (1-based)
```

### stats - Reading statistics

```bash
i-rs-read stats [OPTIONS]

Options:
  --tag <TAG>  Filter statistics by tag
```

## Data Storage

- macOS: `~/.config/i-rs/read.json`
- Linux: `~/.config/i-rs/read.json`
- Windows: `~\AppData\Roaming\i-rs\read.json`

Override with `CONFIG_DIR` environment variable.

## JSON Output

All commands support `--json` global flag:

```bash
i-rs-read list --json
i-rs-read get "Book Name" --json
i-rs-read stats --json
```

## License

MIT OR Apache-2.0
