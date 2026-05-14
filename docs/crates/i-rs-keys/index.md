# i-rs-keys

API keys and secrets management CLI tool using OS keychain for secure storage.

## Overview

i-rs-keys helps you securely store and manage API keys, passwords, and other secrets using your operating system's native keychain.

## Quick Start

```bash
# Add a key
i-rs-keys add "github-token" --type api-key --remark "GitHub personal access token"

# List all keys
i-rs-keys list

# Get key details
i-rs-keys get github-token
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-keys

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-keys
```

## Data Storage

- Keys stored securely in OS Keychain (macOS Keychain, Linux Secret Service, or Windows Credential Manager)
- Metadata in: `~/.config/i-rs/keys.json`

## Features

- **Secure Storage**: Keys stored in OS keychain
- **Key Types**: Support for api-key, aws-key, ssh-key, password, token, other
- **Tag Support**: Categorize keys with tags
- **Quick Access**: Retrieve keys when needed

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records