# i-rs-article

Article read-later CLI tool for saving and organizing articles for future reading.

## Features

- Save articles with title, URL, and source
- Track reading status (unread/reading/read)
- Add reading notes
- Tag support for organization
- Reading statistics
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-article
# or
brew install i-rs/homebrew-tap/i-rs-article
```

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

## Commands

| Command | Description |
|---------|-------------|
| `add` | Add a new article |
| `list` | List articles (with optional filters) |
| `get` | Get article details |
| `read` | Mark article as read |
| `update` | Update article information |
| `delete` | Delete an article |
| `stats` | Show reading statistics |
| `example` | Show usage examples |
| `skill` | View AI skill documentation |

## Data Storage

- macOS: `~/.config/i-rs/articles.json`
- Linux: `~/.config/i-rs/articles.json`
- Windows: `~\AppData\Roaming\i-rs\articles.json`

## License

MIT OR Apache-2.0
