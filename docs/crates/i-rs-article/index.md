# i-rs-article Overview

Article read-later CLI tool for saving and organizing articles for future reading.

## Features

- Save articles with title, URL, and source
- Track reading status (unread/reading/read)
- Add reading notes
- Tag support for organization
- Reading statistics
- JSON output support

## Quick Start

```bash
# Add an article
i-rs-article add rust-blog https://rust-lang.org/blog "Rust Blog" --source rust-lang.org --tag programming

# List all articles
i-rs-article list

# List unread articles
i-rs-article list --status unread

# Mark as reading
i-rs-article update rust-blog --status reading

# Mark as read
i-rs-article read rust-blog

# View statistics
i-rs-article stats
```

## Data Storage

- Config: `~/.config/i-rs/articles.json`
- Override with `CONFIG_DIR` environment variable
