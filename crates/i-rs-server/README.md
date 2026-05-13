# i-rs-server

Server management CLI tool for managing server configurations locally.

## Install

```bash
npm install -g @i-rs/i-rs-server
# or
brew install i-rs/homebrew-tap/i-rs-server
```

## Security

**Passwords are stored securely in the OS keychain, never in the JSON config file.**

Supported keychain backends:
- macOS: Keychain
- Linux: Secret Service / keyutils
- Windows: Credential Manager

## Usage

### Add Server

```bash
i-rs-server add <NAME> <HOST> [PORT] [OPTIONS]
```

Options:
- `-u, --user <USER>` - SSH user
- `-p, --password <PASSWORD>` - SSH password (stored securely in keychain)
- `-t, --tag <TAG>` - Tags (can be specified multiple times)
- `-n, --note <NOTE>` - Notes (can be specified multiple times)

### List Servers

```bash
i-rs-server list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### Get Server Details

```bash
i-rs-server get <NAME> [OPTIONS]
```

Options:
- `-s, --show-password` - Show password from keychain

### Update Server

```bash
i-rs-server update <NAME> [OPTIONS]
```

Options:
- `--host <HOST>` - New host
- `-P, --port <PORT>` - New port
- `-u, --user <USER>` - New user
- `-p, --password <PASSWORD>` - New password (stored securely in keychain)
- `-t, --tag <TAG>` - New tags
- `-n, --note <NOTE>` - New notes

### Delete Server

```bash
i-rs-server delete <NAME>
```

### Suggest Commands

List suggested SSH commands for server management.

```bash
i-rs-server suggest <NAME> [OPTIONS]
```

Options:
- `-c, --command <CMD>` - Filter by keyword (e.g., docker, disk, port, memory)

## Examples

```bash
# Add a production web server (password stored in keychain)
i-rs-server add web1 192.168.1.100 22 --user admin --password secret --tag production --note "Primary web server"

# List all servers
i-rs-server list

# List production servers only
i-rs-server list --tag production

# Get server details (password hidden)
i-rs-server get web1

# Get server details with password visible
i-rs-server get web1 --show-password

# Get suggested commands
i-rs-server suggest web1

# Filter docker commands
i-rs-server suggest web1 --command docker

# Update server
i-rs-server update web1 --tag cloud

# Delete a server
i-rs-server delete web1
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/servers.json`
- Linux: `~/.config/i-rs/servers.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

**Passwords are NEVER stored in the config file. They go to the OS keychain.**

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-server list
```

## License

MIT OR Apache-2.0
