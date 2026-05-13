---
name: "i-rs-domain"
description: "Manages domains (add/list/get/update/delete). Invoke when user needs to track domain names, expiry dates, or renewal countdowns."
---

# i-rs-domain

Domain management CLI tool for managing domain names and expiry tracking.

## Security

Passwords are stored securely in the OS keychain:
- macOS: Keychain
- Linux: Secret Service / keyutils
- Windows: Credential Manager

**Passwords are never stored in the JSON config file.**

## Storage

- Config: `~/.config/i-rs/domains.json`
- Passwords: OS Keychain (never in config file)

## Commands

### add

Add a new domain.

```bash
i-rs-domain add <NAME> <EXPIRY_DATE>
```

Options:
- `-r, --registrar <REGISTRAR>` - Domain registrar (optional)
- `-p, --password <PASSWORD>` - Registrar password (stored securely in keychain, optional)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-m, --remark <REMARK>` - Remarks (can be repeated)

### list

List domains.

```bash
i-rs-domain list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get domain details with renewal countdown.

```bash
i-rs-domain get <NAME>
```

Options:
- `-s, --show-password` - Show password from keychain (default: hidden)

### update

Update domain.

```bash
i-rs-domain update <NAME>
```

Options:
- `-e, --expiry-date <DATE>` - New expiry date (YYYY-MM-DD)
- `-r, --registrar <REGISTRAR>` - New registrar
- `-p, --password <PASSWORD>` - New password (stored securely in keychain)
- `-t, --tag <TAG>` - New tags
- `-m, --remark <REMARK>` - New remarks

### delete

Delete domain.

```bash
i-rs-domain delete <NAME>
```

## Examples

```bash
# Add a domain
i-rs-domain add example.com 2025-12-31 --registrar GoDaddy --tag important

# Add a domain without password
i-rs-domain add github.io 2026-06-15 --tag personal

# List all domains
i-rs-domain list

# Get domain details (shows days until expiry)
i-rs-domain get example.com

# Get domain with password visible
i-rs-domain get example.com --show-password

# Update expiry date
i-rs-domain update example.com --expiry-date 2026-12-31

# Delete domain
i-rs-domain delete example.com
```
