---
name: "i-rs-password"
description: "Manages password entries (add/list/get/update/delete). Invoke when user needs to manage passwords, credentials, or account information."
---

# i-rs-password

Password management CLI tool for storing account credentials securely locally.

## Security

Passwords are stored securely in the OS keychain:
- macOS: Keychain
- Linux: Secret Service / keyutils
- Windows: Credential Manager

**Passwords are never stored in the JSON config file.**

## Storage

- Config: `~/.config/i-rs/passwords.json`
- Passwords: OS Keychain (never in config file)

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add a new password entry.

```bash
i-rs-password add <NAME> <URL> [OPTIONS]
```

Options:
- `-u, --account <ACCOUNT>` - Account/username
- `-p, --password <PASSWORD>` - Password (stored securely in keychain)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List password entries.

```bash
i-rs-password list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get entry details.

```bash
i-rs-password get <NAME>
```

Options:
- `-s, --show-password` - Show password from keychain (default: hidden)

### update

Update password entry.

```bash
i-rs-password update <NAME>
```

Options:
- `--url <URL>` - New URL
- `-u, --account <ACCOUNT>` - New account
- `-p, --password <PASSWORD>` - New password (stored securely in keychain)
- `-t, --tag <TAG>` - New tags
- `-r, --remark <REMARK>` - New remarks

### delete

Delete password entry.

```bash
i-rs-password delete <NAME>
```

### data

Manage data (export, import, clear).

```bash
i-rs-password data export
i-rs-password data import [FILE]
i-rs-password data clear
```

### example

Show usage examples.

```bash
i-rs-password example
```

### skill

Show skill information.

```bash
i-rs-password skill [summary|content|raw]
```

## Examples

```bash
# Add website account (password stored in keychain)
i-rs-password add github https://github.com --account user@example.com --password secret123 --tag work [OPTIONS]

# Add database credential
i-rs-password add db-prod mysql://db.example.com:3306 --account admin --password dbpass --tag production [OPTIONS]

# List all entries
i-rs-password list [OPTIONS]

# List entries by tag
i-rs-password list --tag work

# Get details (password hidden by default)
i-rs-password get github

# Get details with password visible
i-rs-password get github --show-password

# Update password
i-rs-password update github --password newsecret

# Update remarks
i-rs-password update github --remark "Updated"

# Delete entry
i-rs-password delete github
```
