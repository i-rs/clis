# i-rs-password Usage Guide

## Commands Overview

| Command | Description |
|---------|-------------|
| `add` | Add a new password entry |
| `list` | List all password entries |
| `get` | Get entry details |
| `update` | Update a password entry |
| `delete` | Delete a password entry |

## add

Add a new password entry.

```bash
i-rs-password add <NAME> <URL> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Entry name (unique identifier) | Yes |
| `URL` | Website URL or connection string | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-u` | `--account` | Account/username |
| `-p` | `--password` | Password (stored securely in keychain) |
| `-t` | `--tag` | Tags (can be specified multiple times) |
| `-r` | `--remark` | Remarks (can be specified multiple times) |

### Examples

```bash
# Add website account
i-rs-password add github https://github.com --account user@example.com --password secret123 --tag work

# Add database credential
i-rs-password add db-prod mysql://db.example.com:3306 --account admin --password dbpass --tag production

# Add API key
i-rs-password add stripe-api https://api.stripe.com --account api_user --password sk_live_xxx --tag payment
```

---

## list

List all password entries, optionally filtered by tag.

```bash
i-rs-password list [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-t` | `--tag` | Filter by tag |

### Examples

```bash
# List all entries
i-rs-password list

# List entries by tag
i-rs-password list --tag work
i-rs-password list --tag production
```

---

## get

Get detailed information about a specific entry.

```bash
i-rs-password get <NAME> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Entry name | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-s` | `--show-password` | Show password from keychain (default: hidden) |

### Examples

```bash
# Get entry details (password hidden)
i-rs-password get github

# Get entry details with password visible
i-rs-password get github --show-password
```

---

## update

Update an existing password entry.

```bash
i-rs-password update <NAME> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Entry name to update | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| | `--url` | New URL |
| `-u` | `--account` | New account |
| `-p` | `--password` | New password (stored securely in keychain) |
| `-t` | `--tag` | New tags (replaces all existing tags) |
| `-r` | `--remark` | New remarks (replaces all existing remarks) |

### Examples

```bash
# Update URL
i-rs-password update github --url https://github.com

# Update account
i-rs-password update github --account newuser@example.com

# Update password
i-rs-password update github --password newpassword

# Update tags
i-rs-password update github --tag work --tag github

# Update multiple fields
i-rs-password update github --account updated@example.com --password newpass --tag important
```

---

## delete

Delete a password entry from the configuration.

```bash
i-rs-password delete <NAME>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Entry name to delete | Yes |

### Examples

```bash
# Delete an entry
i-rs-password delete github
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CONFIG_DIR` | Override config directory path |

### Examples

```bash
CONFIG_DIR=/tmp i-rs-password list
```

---

## Supported URL Types

i-rs-password supports various types of URLs:

- **Websites**: `https://github.com`, `http://localhost:3000`
- **Databases**: `mysql://host:port`, `postgresql://host:port`, `mongodb://host:port`
- **API Endpoints**: `https://api.stripe.com`, `https://api.sendgrid.com`
- **Custom**: Any valid URL or connection string