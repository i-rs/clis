# i-rs-password Usage Guide

## Install

```bash
npm install -g @i-rs/i-rs-password
# or
brew install i-rs/homebrew-tap/i-rs-password
```

## Security

**Passwords are stored securely in the OS keychain, never in the JSON config file.**

Supported keychain backends:
- macOS: Keychain
- Linux: Secret Service / keyutils
- Windows: Credential Manager

## Commands

### add

Add a new password entry.

```bash
i-rs-password add <NAME> <URL> [OPTIONS]
```

Options:
- `-u, --account <ACCOUNT>` - Account/username
- `-p, --password <PASSWORD>` - Password (stored securely in keychain)
- `-t, --tag <TAG>` - Tags (can be used multiple times)
- `-r, --remark <REMARK>` - Remarks (can be used multiple times)

### list

List all password entries.

```bash
i-rs-password list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get entry details.

```bash
i-rs-password get <NAME> [OPTIONS]
```

Options:
- `-s, --show-password` - Show password from keychain (default: hidden)

### update

Update a password entry.

```bash
i-rs-password update <NAME> [OPTIONS]
```

Options:
- `--url <URL>` - New URL
- `-u, --account <ACCOUNT>` - New account
- `-p, --password <PASSWORD>` - New password (stored securely in keychain)
- `-t, --tag <TAG>` - New tags
- `-r, --remark <REMARK>` - New remarks

### delete

Delete a password entry.

```bash
i-rs-password delete <NAME>
```

## Examples

```bash
# Add a website account (password stored in keychain)
i-rs-password add github https://github.com --account user@example.com --password secret123 --tag work --remark "GitHub account"

# Add a database credential
i-rs-password add db-prod mysql://db.example.com:3306 --account admin --password dbpass --tag production --remark "Production database"

# List all entries
i-rs-password list

# List entries by tag
i-rs-password list --tag work

# Get entry details (password hidden)
i-rs-password get github

# Get entry details with password visible
i-rs-password get github --show-password

# Update entry
i-rs-password update github --remark "Updated credentials"

# Delete an entry
i-rs-password delete github
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/passwords.json`
- Linux: `~/.config/i-rs/passwords.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

**Passwords are NEVER stored in the config file. They go to the OS keychain.**
