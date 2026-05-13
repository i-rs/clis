# i-rs-server Usage Guide

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

## Commands

### add

Add a new server.

```bash
i-rs-server add <NAME> <HOST> [PORT] [OPTIONS]
```

Options:
- `-u, --user <USER>` - SSH username
- `-p, --password <PASSWORD>` - SSH password (stored securely in keychain)
- `-t, --tag <TAG>` - Tags (can be used multiple times)
- `-n, --note <NOTE>` - Notes (can be used multiple times)

### list

List all servers.

```bash
i-rs-server list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get server details.

```bash
i-rs-server get <NAME>
```

### update

Update a server.

```bash
i-rs-server update <NAME> [OPTIONS]
```

Options:
- `--host <HOST>` - New host
- `-P, --port <PORT>` - New port
- `-u, --user <USER>` - New username
- `-p, --password <PASSWORD>` - New password (stored securely in keychain)
- `-t, --tag <TAG>` - New tags
- `-n, --note <NOTE>` - New notes

### delete

Delete a server.

```bash
i-rs-server delete <NAME>
```

### suggest

List suggested SSH commands for server management.

```bash
i-rs-server suggest <NAME> [OPTIONS]
```

Options:
- `-c, --command <CMD>` - Filter by keyword

## Command Categories

### SSH Connection
- `ssh` - Direct SSH login
- `ssh_key` - Setup passwordless SSH

### File Transfer
- `scp_up` - Upload file via SCP
- `scp_down` - Download file via SCP

### Disk & Storage
- `disk` - Disk usage (df -h)
- `disk_inode` - Inode usage (df -i)
- `disk_large` - Find large directories

### CPU & Process
- `cpu` - CPU info and load
- `proc_mem` - Top memory processes
- `proc_cpu` - Top CPU processes
- `process_tree` - Process tree

### Memory
- `memory` - Memory usage (free -h)

### Network
- `port` - Port usage (ss)
- `port_netstat` - Port usage (netstat)
- `listen_port` - Find process on port
- `connection` - Network connections
- `sysload` - System load (vmstat)
- `iostat` - IO statistics
- `sar_net` - Network stats (sar)
- `tcpdump` - Capture traffic

### System Info
- `sys_info` - System information
- `uptime` - System uptime
- `who` - Logged in users
- `last` - Recent logins
- `sysctl` - Kernel parameters
- `limits` - User limits
- `killed_procs` - OOM killed processes

### Services
- `service` - Running services
- `service_status` - Check service status
- `journal` - Systemd journal

### Docker
- `docker_ps` - Docker containers
- `docker_psa` - All containers
- `docker_images` - Docker images
- `docker_logs` - Container logs
- `docker_stats` - Docker stats
- `docker_cleanup` - Docker cleanup

### Logs
- `nginx_access` - Nginx access log
- `nginx_error` - Nginx error log
- `syslog` - System logs
- `auth_log` - Auth logs (failed login)

### Security
- `cron` - Cron jobs
- `firewall` - Firewall status (ufw)
- `firewall_iptables` - iptables rules

### Config
- `mount` - Mount points
- `fstab` - Fstab config
- `dns` - DNS configuration
- `hosts` - Hosts file

## Examples

```bash
# Add server (password stored in keychain)
i-rs-server add web1 192.168.1.100 22 --user admin --password secret --tag production

# List all servers
i-rs-server list

# List with tag filter
i-rs-server list --tag production

# Get server details
i-rs-server get web1

# Get all suggested commands
i-rs-server suggest web1

# Filter docker commands
i-rs-server suggest web1 --command docker

# Filter disk commands
i-rs-server suggest web1 --command disk

# Update server
i-rs-server update web1 --tag cloud --user ubuntu

# Update password
i-rs-server update web1 --password newpassword

# Delete server
i-rs-server delete web1
```

## Data Storage

- macOS: `~/.config/i-rs/servers.json`
- Linux: `~/.config/i-rs/servers.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

**Passwords are NEVER stored in the config file. They go to the OS keychain.**
