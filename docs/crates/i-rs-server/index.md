# i-rs-server

Server management CLI tool for managing server configurations locally.

## Overview

i-rs-server helps you manage SSH server configurations with secure password storage in your OS keychain. It provides commands to add, list, get, update, delete servers, and even suggests useful SSH commands for server management.

## Quick Start

```bash
# Add a server
i-rs-server add web1 192.168.1.100 22 --user admin --password secret --tag production --remark "Primary web server"

# List all servers
i-rs-server list

# List production servers
i-rs-server list --tag production

# Get server details
i-rs-server get web1

# Get server with password visible
i-rs-server get web1 --show-password

# Get suggested SSH commands
i-rs-server suggest web1

# Filter docker commands
i-rs-server suggest web1 --command docker

# Update server
i-rs-server update web1 --tag cloud --user ubuntu

# Delete server
i-rs-server delete web1
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-server

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-server
```

## Security

**Passwords are stored securely in the OS keychain, never in the JSON config file.**

Supported keychain backends:
- macOS: Keychain
- Linux: Secret Service / keyutils
- Windows: Credential Manager

## Data Storage

- macOS: `~/.config/i-rs/servers.json`
- Linux: `~/.config/i-rs/servers.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

**Passwords are NEVER stored in the config file. They go to the OS keychain.**

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records