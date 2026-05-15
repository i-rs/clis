---
name: "i-rs-bookmark"
description: "Manages bookmarks (add/list/get/update/delete). Invoke when user needs to manage bookmarks, URLs, or website credentials."
---

# i-rs-bookmark

Bookmark management CLI tool for managing URLs and credentials locally.

## Security

Passwords are stored securely in the OS keychain:
- macOS: Keychain
- Linux: Secret Service / keyutils
- Windows: Credential Manager

**Passwords are never stored in the JSON config file.**

## Storage

- Config: `~/.config/i-rs/bookmarks.json`
- Passwords: OS Keychain (never in config file)

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add a new bookmark.

```bash
i-rs-bookmark add <NAME> <URL>
```

Options:
- `-u, --account <ACCOUNT>` - Account/username (optional)
- `-p, --password <PASSWORD>` - Password (stored securely in keychain, optional)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List bookmarks.

```bash
i-rs-bookmark list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get bookmark details.

```bash
i-rs-bookmark get <NAME>
```

Options:
- `-s, --show-password` - Show password from keychain (default: hidden)

### update

Update bookmark.

```bash
i-rs-bookmark update <NAME>
```

Options:
- `--url <URL>` - New URL
- `-u, --account <ACCOUNT>` - New account
- `-p, --password <PASSWORD>` - New password (stored securely in keychain)
- `-t, --tag <TAG>` - New tags
- `-r, --remark <REMARK>` - New remarks

### delete

Delete bookmark.

```bash
i-rs-bookmark delete <NAME>
```

### data

Manage data (export, import, clear).

```bash
i-rs-bookmark data export
i-rs-bookmark data import [FILE]
i-rs-bookmark data clear
```

### example

Show usage examples.

```bash
i-rs-bookmark example
```

### skill

Show skill information.

```bash
i-rs-bookmark skill [summary|content|raw]
```

## Examples

```bash
# Add a simple bookmark (no account/password)
i-rs-bookmark add github https://github.com --tag work --remark "GitHub"

# Add a bookmark with credentials
i-rs-bookmark add aws https://aws.amazon.com --account admin@example.com --password secret123 --tag cloud

# List all bookmarks
i-rs-bookmark list

# List bookmarks by tag
i-rs-bookmark list --tag work

# Get details (password hidden by default)
i-rs-bookmark get github

# Get details with password visible
i-rs-bookmark get aws --show-password

# Update bookmark
i-rs-bookmark update github --remark "Updated"

# Delete bookmark
i-rs-bookmark delete github
```
