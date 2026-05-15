# i-rs-server Usage Guide

## Global Flags

- `--json` — Output in JSON format

## Commands Overview

| Command | Description |
|---------|-------------|
| `add` | Add a new server |
| `list` | List all servers |
| `get` | Get server details |
| `update` | Update a server |
| `delete` | Delete a server |
| `suggest` | List suggested SSH commands |

## add

Add a new server to the configuration.

```bash
i-rs-server add <NAME> <HOST> [PORT] [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Server name (unique identifier) | Yes |
| `HOST` | Server hostname or IP address | Yes |
| `PORT` | SSH port (default: 22) | No |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-u` | `--user` | SSH username |
| `-p` | `--password` | SSH password (stored securely in keychain) |
| `-t` | `--tag` | Tags (can be specified multiple times) |
| `-r` | `--remark` | Remarks (can be specified multiple times) |

### Examples

```bash
# Add server with minimal options
i-rs-server add web1 192.168.1.100

# Add server with custom port
i-rs-server add web1 192.168.1.100 2222

# Add server with full options
i-rs-server add prod-server 203.0.113.50 22 --user admin --password secret --tag production --tag web --remark "Primary production server"
```

---

## list

List all servers, optionally filtered by tag.

```bash
i-rs-server list [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-t` | `--tag` | Filter by tag |

### Examples

```bash
# List all servers
i-rs-server list

# List servers with specific tag
i-rs-server list --tag production

# List servers with multiple tags (shows servers matching ANY tag)
i-rs-server list --tag web --tag database
```

---

## get

Get detailed information about a specific server.

```bash
i-rs-server get <NAME> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Server name | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-s` | `--show-password` | Show password from keychain (default: hidden) |

### Examples

```bash
# Get server details (password hidden)
i-rs-server get web1

# Get server details with password visible
i-rs-server get web1 --show-password

# Get server details for a specific server
i-rs-server get prod-server
```

---

## update

Update an existing server's configuration.

```bash
i-rs-server update <NAME> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Server name to update | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| | `--host` | New hostname or IP address |
| `-P` | `--port` | New SSH port |
| `-u` | `--user` | New SSH username |
| `-p` | `--password` | New password (stored securely in keychain) |
| `-t` | `--tag` | New tags (replaces all existing tags) |
| `-r` | `--remark` | New remarks (replaces all existing remarks) |

### Examples

```bash
# Update server host
i-rs-server update web1 --host 192.168.1.101

# Update port and user
i-rs-server update web1 -P 2222 -u ubuntu

# Update password
i-rs-server update web1 --password newpassword

# Update tags (replaces all existing tags)
i-rs-server update web1 --tag production --tag web

# Update multiple fields
i-rs-server update web1 --host 10.0.0.1 -u root --tag cloud --remark "Updated configuration"
```

---

## delete

Delete a server from the configuration.

```bash
i-rs-server delete <NAME>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Server name to delete | Yes |

### Examples

```bash
# Delete a server
i-rs-server delete web1

# Delete multiple servers (run command multiple times)
i-rs-server delete old-server
i-rs-server delete test-server
```

---

## suggest

List suggested SSH commands for server management. Commands are grouped by category.

```bash
i-rs-server suggest <NAME> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Server name | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-c` | `--command` | Filter by keyword (e.g., docker, disk, port, memory) |

### Command Categories

#### SSH Connection
| Command | Description |
|---------|-------------|
| `ssh` | Direct SSH login |
| `ssh_key` | Setup passwordless SSH |

#### File Transfer
| Command | Description |
|---------|-------------|
| `scp_up` | Upload file via SCP |
| `scp_down` | Download file via SCP |

#### Disk & Storage
| Command | Description |
|---------|-------------|
| `disk` | Disk usage (df -h) |
| `disk_inode` | Inode usage (df -i) |
| `disk_large` | Find large directories |

#### CPU & Process
| Command | Description |
|---------|-------------|
| `cpu` | CPU info and load |
| `proc_mem` | Top memory processes |
| `proc_cpu` | Top CPU processes |
| `process_tree` | Process tree |

#### Memory
| Command | Description |
|---------|-------------|
| `memory` | Memory usage (free -h) |

#### Network
| Command | Description |
|---------|-------------|
| `port` | Port usage (ss) |
| `port_netstat` | Port usage (netstat) |
| `listen_port` | Find process on port |
| `connection` | Network connections |
| `sysload` | System load (vmstat) |
| `iostat` | IO statistics |
| `sar_net` | Network stats (sar) |
| `tcpdump` | Capture traffic |

#### System Info
| Command | Description |
|---------|-------------|
| `sys_info` | System information |
| `uptime` | System uptime |
| `who` | Logged in users |
| `last` | Recent logins |
| `sysctl` | Kernel parameters |
| `limits` | User limits |
| `killed_procs` | OOM killed processes |

#### Services
| Command | Description |
|---------|-------------|
| `service` | Running services |
| `service_status` | Check service status |
| `journal` | Systemd journal |

#### Docker
| Command | Description |
|---------|-------------|
| `docker_ps` | Docker containers |
| `docker_psa` | All containers |
| `docker_images` | Docker images |
| `docker_logs` | Container logs |
| `docker_stats` | Docker stats |
| `docker_cleanup` | Docker cleanup |

#### Logs
| Command | Description |
|---------|-------------|
| `nginx_access` | Nginx access log |
| `nginx_error` | Nginx error log |
| `syslog` | System logs |
| `auth_log` | Auth logs (failed login) |

#### Security
| Command | Description |
|---------|-------------|
| `cron` | Cron jobs |
| `firewall` | Firewall status (ufw) |
| `firewall_iptables` | iptables rules |

#### Config
| Command | Description |
|---------|-------------|
| `mount` | Mount points |
| `fstab` | Fstab config |
| `dns` | DNS configuration |
| `hosts` | Hosts file |

### Examples

```bash
# Get all suggested commands
i-rs-server suggest web1

# Filter docker commands only
i-rs-server suggest web1 --command docker

# Filter disk commands only
i-rs-server suggest web1 --command disk

# Filter network commands only
i-rs-server suggest web1 --command port

# Filter memory commands only
i-rs-server suggest web1 --command memory
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CONFIG_DIR` | Override config directory path |

### Examples

```bash
# Use custom config directory
CONFIG_DIR=/tmp i-rs-server list

# Use custom config directory for testing
CONFIG_DIR=/tmp/test-config i-rs-server add test 192.168.1.1
```

---

## Error Handling

The tool uses `anyhow::Result<()>` for error handling. Common errors include:

- **Server not found**: The specified server name doesn't exist
- **Duplicate name**: A server with the same name already exists
- **Keychain error**: Unable to access the OS keychain
- **Permission denied**: Unable to read/write the config file

Exit codes:
- `0`: Success
- `1`: Error (with error message printed to stderr)

### data

Manage data (export, import, clear).

```bash
i-rs-server data export
i-rs-server data import [FILE]
i-rs-server data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-server example
```
### skill

Show skill information.

```bash
i-rs-server skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/server.json`
- Linux: `~/.config/i-rs/server.json`
- Windows: `~\AppData\Roaming\i-rs\server.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-server list
```
