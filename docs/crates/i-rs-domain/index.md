# i-rs-domain

Domain management CLI tool for managing domain names and expiry tracking.

## Overview

i-rs-domain helps you track domain registrations, expiry dates, and registrar information. Supports storing registrar credentials securely in OS keychain.

## Quick Start

```bash
# Add a domain
i-rs-domain add example.com 2025-12-31 --registrar GoDaddy --tag important --remark "Primary domain"

# Add domain without password
i-rs-domain add github.io 2026-06-15 --tag personal

# List domains
i-rs-domain list

# List by tag
i-rs-domain list --tag important

# Get domain details
i-rs-domain get example.com

# Get with password visible
i-rs-domain get example.com --show-password

# Update expiry date
i-rs-domain update example.com --expiry-date 2026-12-31

# Delete domain
i-rs-domain delete example.com
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-domain

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-domain
```

## Security

**Registrar passwords are stored securely in the OS keychain, never in the JSON config file.**

## Data Storage

- macOS: `~/.config/i-rs/domains.json`
- Linux: `~/.config/i-rs/domains.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records