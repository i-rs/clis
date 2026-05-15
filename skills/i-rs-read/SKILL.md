---
name: "i-rs-read"
description: "Reading progress tracking CLI tool for managing books, tracking reading progress, adding ratings and reviews. Invoke when user needs to track reading plans, manage reading lists, or record reading progress."
---

# i-rs-read

## Global Flags

- `--json` — Output in JSON format

i-rs-read is a reading progress tracking CLI tool that helps users manage books, track reading progress, add ratings and reviews.

## Storage

- **Config file**: `~/.config/i-rs/read.json`
- **Environment**: `CONFIG_DIR` overrides default path

## Commands

### add

Add a new book.

```bash
i-rs-read add <NAME> <AUTHOR> <TOTAL_PAGES> [OPTIONS]

Options:
  --tags <TAGS>       Add tags
  --remark <REMARK>   Add remarks
```

### list

List all books.

```bash
i-rs-read list [OPTIONS]

Options:
  --tag <TAG>         Filter by tag
  --status <STATUS>   Filter by status
```

### get

Get book details.

```bash
i-rs-read get <NAME>
```

### update

Update book information.

```bash
i-rs-read update <NAME> [OPTIONS]

Options:
  --current-page <PAGE>   Update current page
  --status <STATUS>       Update status (reading, completed, paused, dropped, to_read)
  --rating <RATING>       Add rating (0-5)
  --review <REVIEW>       Add review
  --tags <TAGS>           Update tags
  --add-remark <REMARK>   Add remarks
  --remove-remark <INDEX> Remove remarks
```

### delete

Delete a book.

```bash
i-rs-read delete <NAME>
```

### stats

Show reading statistics.

```bash
i-rs-read stats [OPTIONS]

Options:
  --tag <TAG>  Filter statistics by tag
```

### example

Show usage examples.

```bash
i-rs-read example
```

### skill

View AI skill documentation.

```bash
i-rs-read skill [--summary] [--content]
```

## Reading Status

| Status | Description |
|--------|-------------|
| `to_read` | To read |
| `reading` | Currently reading |
| `completed` | Finished |
| `paused` | Paused |
| `dropped` | Abandoned |

## Data Structure

```json
{
  "books": {
    "Book Name": {
      "name": "Book Name",
      "author": "Author",
      "total_pages": 500,
      "current_page": 250,
      "status": "reading",
      "rating": 4.5,
      "review": "Review content",
      "tags": ["tag1", "tag2"],
      "remark": ["remark1", "remark2"],
      "created_at": 1234567890,
      "updated_at": 1234567890
    }
  }
}
```

## Examples

```bash
# Add a book
i-rs-read add "The Rust Programming Language" "Steve Klabnik" 500

# Update reading progress
i-rs-read update "The Rust Programming Language" --current-page 250

# Mark as completed with rating
i-rs-read update "The Rust Programming Language" --status completed --rating 5

# List all books
i-rs-read list

# Filter by tag
i-rs-read list --tag programming

# View statistics
i-rs-read stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-read data export
i-rs-read data import [FILE]
i-rs-read data clear
```

## Examples

```bash
# JSON output
i-rs-read list --json
```
