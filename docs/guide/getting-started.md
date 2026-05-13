# Getting Started

## Installation

### Prerequisites

- Rust 1.75+ (for building from source)
- Node.js 20+ (for npm installation)

### Install via npm

```bash
npm install -g @i-rs/i-rs-server @i-rs/i-rs-password @i-rs/i-rs-bookmark @i-rs/i-rs-note @i-rs/i-rs-domain @i-rs/i-rs-remind @i-rs/i-rs-weight
```

### Install via Homebrew

```bash
brew install i-rs/homebrew-tap/i-rs-server
brew install i-rs/homebrew-tap/i-rs-password
brew install i-rs/homebrew-tap/i-rs-bookmark
brew install i-rs/homebrew-tap/i-rs-note
brew install i-rs/homebrew-tap/i-rs-domain
brew install i-rs/homebrew-tap/i-rs-remind
brew install i-rs/homebrew-tap/i-rs-weight
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/i-rs/clis.git
cd clis

# Build all tools
cargo build

# Or build a specific tool
cargo build -p i-rs-server
```

## Quick Start

### i-rs-server

```bash
# Add a server
i-rs-server add prod-1 192.168.1.100 22 --user admin --password secret --tag production

# List all servers
i-rs-server list

# Get server details
i-rs-server get prod-1

# Get suggested SSH commands
i-rs-server suggest prod-1
```

### i-rs-password

```bash
# Add a password entry
i-rs-password add github https://github.com --account user@example.com --password secret123 --tag work

# List all entries
i-rs-password list

# Get entry with password visible
i-rs-password get github --show-password
```

### i-rs-bookmark

```bash
# Add a bookmark
i-rs-bookmark add aws https://aws.amazon.com --account admin --password secret --tag cloud

# List bookmarks
i-rs-bookmark list --tag cloud
```

### i-rs-note

```bash
# Add a note
i-rs-note add todo --title "My Tasks" --tag work --content "Buy groceries" --content "Call mom"

# List notes
i-rs-note list
```

### i-rs-domain

```bash
# Add a domain
i-rs-domain add example.com 2025-12-31 --registrar GoDaddy --tag important

# List domains
i-rs-domain list
```

### i-rs-remind

```bash
# Add a reminder
i-rs-remind add meeting 2025-06-15 14:00 --title "Team Meeting" --tag work

# List reminders
i-rs-remind list

# Mark as done
i-rs-remind done meeting
```

### i-rs-weight

```bash
# Add weight record
i-rs-weight add 2025-01-15 70.5

# List with chart and stats
i-rs-weight list --days 30 --chart --stats
```

## Data Storage

All tools store data locally in `~/.config/i-rs/`:

| Tool | Data File |
|------|-----------|
| i-rs-server | servers.json |
| i-rs-password | passwords.json |
| i-rs-bookmark | bookmarks.json |
| i-rs-note | notes.json |
| i-rs-domain | domains.json |
| i-rs-remind | reminds.json |
| i-rs-weight | weights.json |

### Custom Data Directory

Override the default storage location using the `CONFIG_DIR` environment variable:

```bash
CONFIG_DIR=/tmp/custom-config i-rs-server list
```

## Security

Passwords for the following tools are stored in the OS keychain, never in JSON config files:
- i-rs-server (SSH passwords)
- i-rs-password (account passwords)
- i-rs-bookmark (website credentials)
- i-rs-domain (registrar passwords)

Supported keychain backends:
- macOS: Keychain
- Linux: Secret Service / keyutils
- Windows: Credential Manager

## Development

```bash
# Build all crates
cargo build

# Run specific tool
cargo run -p i-rs-server -- --help

# Test specific crate
cargo test -p i-rs-server
```

## Release

```bash
# Update version in Cargo.toml
git tag v0.0.x
git push origin v0.0.x
```

CI will automatically build and publish to GitHub Releases, npm, and Homebrew.