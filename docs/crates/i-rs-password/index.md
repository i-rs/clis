# i-rs-password

Password management CLI tool for storing account credentials securely locally.

## Overview

i-rs-password helps you manage passwords and account credentials with secure storage in your OS keychain. Store website logins, database credentials, API keys, and more.

## Quick Start

```bash
# Add a password entry
i-rs-password add github https://github.com --account user@example.com --password secret123 --tag work --remark "GitHub account"

# Add database credentials
i-rs-password add db-prod mysql://db.example.com:3306 --account admin --password dbpass --tag production --remark "Production database"

# List all entries
i-rs-password list

# List entries by tag
i-rs-password list --tag work

# Get entry with password visible
i-rs-password get github --show-password

# Update password
i-rs-password update github --password newpassword

# Delete entry
i-rs-password delete github
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-password

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-password
```

## Security

**Passwords are stored securely in the OS keychain, never in the JSON config file.**

Supported keychain backends:
- macOS: Keychain
- Linux: Secret Service / keyutils
- Windows: Credential Manager

## Data Storage

- macOS: `~/.config/i-rs/passwords.json`
- Linux: `~/.config/i-rs/passwords.json`
- Windows: `~\AppData\Roaming\i-rs\password.json`

**Passwords are NEVER stored in the config file. They go to the OS keychain.**

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records