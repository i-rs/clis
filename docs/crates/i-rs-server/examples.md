# i-rs-server Examples

This page provides comprehensive examples for all i-rs-server commands.

## Basic Server Management

### Adding Your First Server

```bash
# Minimal server addition (uses default port 22)
i-rs-server add web1 192.168.1.100

# With custom SSH port
i-rs-server add web1 192.168.1.100 2222

# With username
i-rs-server add web1 192.168.1.100 --user admin

# With username and password (stored in keychain)
i-rs-server add web1 192.168.1.100 --user admin --password mysecretpassword

# With tags for organization
i-rs-server add web1 192.168.1.100 --user admin --tag production --tag web

# With remarks
i-rs-server add web1 192.168.1.100 --user admin --remark "Primary web server" --remark "Deployed in 2024"
```

### Adding Multiple Servers

```bash
# Production web server
i-rs-server add prod-web-1 10.0.1.10 22 \
  --user admin \
  --password secret \
  --tag production \
  --tag web \
  --tag nginx

# Database server
i-rs-server add prod-db-1 10.0.2.10 22 \
  --user dbadmin \
  --password dbpass123 \
  --tag production \
  --tag database \
  --tag mysql

# Staging server
i-rs-server add staging-1 10.0.3.10 22 \
  --user ubuntu \
  --tag staging \
  --tag web

# Development server
i-rs-server add dev-1 192.168.1.50 22 \
  --user developer \
  --password devpass \
  --tag development \
  --tag docker
```

### Listing Servers

```bash
# List all servers
i-rs-server list

# Filter by tag - production servers
i-rs-server list --tag production

# Filter by tag - web servers
i-rs-server list --tag web

# Filter by tag - database servers
i-rs-server list --tag database
```

### Getting Server Details

```bash
# View server details (password hidden by default)
i-rs-server get web1

# View server with password visible
i-rs-server get web1 --show-password

# View specific server details
i-rs-server get prod-web-1
```

### Updating Servers

```bash
# Update just the tags
i-rs-server update web1 --tag production --tag web --tag ssl

# Update username
i-rs-server update web1 --user ubuntu

# Update password (stored securely in keychain)
i-rs-server update web1 --password newpassword123

# Update host and port
i-rs-server update web1 --host 192.168.1.101 --port 2222

# Full update
i-rs-server update web1 \
  --host 10.0.0.1 \
  --port 22 \
  --user root \
  --password newpass \
  --tag production \
  --tag updated \
  --remark "Migrated to new IP"
```

### Deleting Servers

```bash
# Delete a single server
i-rs-server delete web1

# Delete multiple servers
i-rs-server delete old-server-1
i-rs-server delete old-server-2
i-rs-server delete test-server
```

## Using SSH Command Suggestions

### Basic Usage

```bash
# Get all available SSH commands for a server
i-rs-server suggest prod-web-1

# Filter commands by category
i-rs-server suggest prod-web-1 --command docker
i-rs-server suggest prod-web-1 --command disk
i-rs-server suggest prod-web-1 --command port
i-rs-server suggest prod-web-1 --command memory
i-rs-server suggest prod-web-1 --command cpu
```

### Command Categories in Detail

#### Docker Management

```bash
# List all docker-related commands
i-rs-server suggest prod-web-1 --command docker
```

Typical output includes:
- `docker_ps` - List running containers
- `docker_psa` - List all containers (including stopped)
- `docker_images` - List Docker images
- `docker_logs` - View container logs
- `docker_stats` - View container resource usage
- `docker_cleanup` - Clean up unused resources

#### Disk Management

```bash
# List all disk-related commands
i-rs-server suggest prod-web-1 --command disk
```

Typical output includes:
- `disk` - Show disk usage
- `disk_inode` - Show inode usage
- `disk_large` - Find large directories

#### Network Analysis

```bash
# List network-related commands
i-rs-server suggest prod-web-1 --command port
```

Typical output includes:
- `port` - Show listening ports
- `listen_port` - Find which process is using a port
- `connection` - Show network connections

## Real-World Scenarios

### Setting Up a New Project Environment

```bash
# Add web server
i-rs-server add web-prod-1 203.0.113.10 22 \
  --user ubuntu \
  --tag production \
  --tag web \
  --tag nginx \
  --tag letsencrypt

# Add database server
i-rs-server add db-prod-1 203.0.113.20 22 \
  --user admin \
  --password securepassword \
  --tag production \
  --tag database \
  --tag postgresql

# Add backup server
i-rs-server add backup-prod-1 203.0.113.30 22 \
  --user backup \
  --tag production \
  --tag backup \
  --tag rsync

# List all production servers
i-rs-server list --tag production
```

### Migrating Servers

```bash
# Before migration - note current configuration
i-rs-server get old-server

# Add new server with migrated IP
i-rs-server add old-server 10.0.0.50 \
  --user ubuntu \
  --tag production \
  --tag migrated \
  --remark "Migrated from old-server on 2024-01-15"

# List migrated servers
i-rs-server list --tag migrated
```

### Monitoring Multiple Servers

```bash
# Add monitoring server
i-rs-server add monitor-1 10.5.0.1 22 \
  --user admin \
  --tag monitoring \
  --tag prometheus

# Add log server
i-rs-server add logs-1 10.5.0.2 22 \
  --user admin \
  --tag logging \
  --tag elk

# Add metrics server
i-rs-server add metrics-1 10.5.0.3 22 \
  --user admin \
  --tag metrics \
  --tag grafana
```

### Using with SSH Config

```bash
# Get SSH command for server
i-rs-server suggest web1 --command ssh
# Output: ssh admin@192.168.1.100

# Use SSH with key-based auth
i-rs-server suggest web1 --command ssh_key

# Copy files to server
i-rs-server suggest web1 --command scp_up
```

## Integration Examples

### Scripting with the Tool

```bash
#!/bin/bash
# Script to backup all production server configs

echo "=== Production Servers ==="
i-rs-server list --tag production

echo ""
echo "=== Database Servers ==="
i-rs-server list --tag database

echo ""
echo "=== Web Servers ==="
i-rs-server list --tag web
```

### Using with SSH Agent

```bash
# First, ensure SSH agent is running
eval "$(ssh-agent -s)"

# Add your key
ssh-add ~/.ssh/id_rsa

# Then SSH to server (password stored in keychain for the tool)
ssh admin@$(i-rs-server get web1 --format json | jq -r '.host')
```

## Troubleshooting Examples

### Server Not Found

```bash
# Error: Server "web1" not found
# Solution: Check existing servers
i-rs-server list

# Or search by tag
i-rs-server list --tag production
```

### Keychain Access Issues

```bash
# If password can't be stored/retrieved from keychain:
# macOS: Open Keychain Access and check if app has access
# Linux: Install libsecret or keyutils
# Windows: Run as administrator for first access
```

### Permission Issues

```bash
# If unable to read/write config:
# Check CONFIG_DIR permissions
ls -la ~/.config/i-rs/

# Or use custom directory with proper permissions
CONFIG_DIR=/tmp/my-config i-rs-server list