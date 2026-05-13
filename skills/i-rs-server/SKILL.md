---
name: "i-rs-server"
description: "Manages server configurations (add/list/get/update/delete/suggest). Invoke when user needs to manage server inventory or retrieve server connection details."
---

# i-rs-server

Server management CLI tool for managing server configurations locally.

## Security

Passwords are stored securely in the OS keychain:

- macOS: Keychain
- Linux: Secret Service / keyutils
- Windows: Credential Manager

**Passwords are never stored in the JSON config file.**

## Storage

- Config: `~/.config/i-rs/servers.json`
- Passwords: OS Keychain (never in config file)

## Commands

### add

Add a new server.

```bash
i-rs-server add <NAME> <HOST> [PORT]
```

Options:

- `-u, --user <USER>` - SSH username
- `-p, --password <PASSWORD>` - SSH password (stored securely in keychain)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-n, --note <NOTE>` - Notes (can be repeated)

### list

List servers.

```bash
i-rs-server list
```

Options:

- `-t, --tag <TAG>` - Filter by tag

### get

Get server details.

```bash
i-rs-server get <NAME>
```

### update

Update server.

```bash
i-rs-server update <NAME>
```

Options:

- `--host <HOST>` - New host
- `-P, --port <PORT>` - New port
- `-u, --user <USER>` - New username
- `-p, --password <PASSWORD>` - New password (stored securely in keychain)
- `-t, --tag <TAG>` - New tags
- `-n, --note <NOTE>` - New notes

### delete

Delete server.

```bash
i-rs-server delete <NAME>
```

### suggest

List suggested SSH commands for a server.

```bash
i-rs-server suggest <NAME>
```

Options:

- `-c, --command <CMD>` - Filter by keyword (e.g., docker, disk, port)

## Examples

```bash
# Add server (password stored in keychain)
i-rs-server add web1 192.168.1.100 22 --user admin --password secret --tag production

# List servers
i-rs-server list

# Get details (password shown as stored in keychain)
i-rs-server get web1

# Update password
i-rs-server update web1 --password newsecret

# Delete
i-rs-server delete web1

# Get suggested commands
i-rs-server suggest web1

# Filter suggested commands
i-rs-server suggest web1 --command docker
```
