# i-rs-read - Reading Progress Tracker

i-rs-read is a lightweight CLI tool for tracking and managing your reading progress. Whether you read technical books, novels, or magazines, i-rs-read helps you record reading status, track progress, add ratings and reviews.

## Overview

i-rs-read provides the following core features:

- **Book Management** - Add, delete, view book information
- **Progress Tracking** - Record current page, auto-calculate completion percentage
- **Status Management** - Multiple reading statuses (reading, completed, paused, dropped, to_read)
- **Ratings & Reviews** - Rate and review completed books
- **Tag Organization** - Use tags to organize books
- **Reading Statistics** - View total books, pages, average rating, etc.

## Quick Start

### Install

```bash
npm install -g @i-rs/i-rs-read
# or
brew install i-rs/homebrew-tap/i-rs-read
```

### Basic Usage

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
| [add](./usage.html#add) | Add a new book |
| [list](./usage.html#list) | List all books |
| [get](./usage.html#get) | Get book details |
| [update](./usage.html#update) | Update book information |
| [delete](./usage.html#delete) | Delete a book |
| [stats](./usage.html#stats) | Show reading statistics |
| [example](./usage.html#example) | Show usage examples |
| [skill](./usage.html#skill) | Show AI skill documentation |

## Reading Status

| Status | Description |
|--------|-------------|
| `to_read` | To read |
| `reading` | Currently reading |
| `completed` | Completed |
| `paused` | Paused |
| `dropped` | Abandoned |

## Data Storage

Book data is stored in local JSON files:

- **macOS**: `~/.config/i-rs/read.json`
- **Linux**: `~/.config/i-rs/read.json`
- **Windows**: `~\AppData\Roaming\i-rs\read.json`

Override with `CONFIG_DIR` environment variable.
