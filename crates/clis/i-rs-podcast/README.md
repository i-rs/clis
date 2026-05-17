# i-rs-podcast

Podcast and course tracking CLI tool for managing your audio/video learning content.

## Features

- Track podcasts and courses with title, author, and duration
- Track listening progress (current position)
- Add notes during listening sessions
- Filter by status (not_started, in_progress, completed)
- Tag support for organization
- Statistics overview with progress percentage
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-podcast
# or
brew install i-rs/homebrew-tap/i-rs-podcast
```

## Quick Start

```bash
# Add a podcast
i-rs-podcast add "The Daily" --author "NYT" --duration 3600

# Update listening progress
i-rs-podcast listen "The Daily" --position 1800

# List all podcasts
i-rs-podcast list

# Show statistics
i-rs-podcast stats
```

## Commands

| Command | Description |
|---------|-------------|
| `add` | Add a new podcast/course |
| `list` | List all podcasts |
| `get` | Show podcast details |
| `listen` | Update listening progress |
| `update` | Update podcast info |
| `delete` | Delete a podcast |
| `stats` | Show statistics |
| `example` | Show usage examples |
| `skill` | Show AI skill documentation |

## Status Icons

- `○` Not Started
- `◐` In Progress
- `●` Completed

## Data Storage

- macOS: `~/.config/i-rs/podcasts.json`
- Linux: `~/.config/i-rs/podcasts.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0
