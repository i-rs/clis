# i-rs-bookmark

Bookmark management CLI tool for managing URLs and credentials locally.

## Overview

i-rs-bookmark helps you manage bookmarks with optional credential storage. Great for storing website URLs with login information, API documentation links, and reference materials.

## Quick Start

```bash
# Add a simple bookmark
i-rs-bookmark add github https://github.com --tag work --remark "GitHub"

# Add bookmark with credentials
i-rs-bookmark add aws https://aws.amazon.com --account admin@example.com --password secret123 --tag cloud --remark "AWS account"

# List bookmarks
i-rs-bookmark list

# List by tag
i-rs-bookmark list --tag work

# Get bookmark with password
i-rs-bookmark get aws --show-password

# Update bookmark
i-rs-bookmark update github --remark "Updated GitHub link"

# Delete bookmark
i-rs-bookmark delete github
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-bookmark

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-bookmark
```

## Security

**Passwords are stored securely in the OS keychain, never in the JSON config file.**

## Data Storage

- macOS: `~/.config/i-rs/bookmarks.json`
- Linux: `~/.config/i-rs/bookmarks.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records