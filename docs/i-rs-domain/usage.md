# i-rs-domain Usage Guide

## Install

```bash
npm install -g @i-rs/i-rs-domain
# or
brew install i-rs/homebrew-tap/i-rs-domain
```

## Commands

### add

Add a new domain.

```bash
i-rs-domain add <NAME> <EXPIRY_DATE> [OPTIONS]
```

Options:
- `-r, --registrar <REGISTRAR>` - Domain registrar (optional)
- `-p, --password <PASSWORD>` - Registrar password (stored securely in keychain, optional)
- `-t, --tag <TAG>` - Tags (can be used multiple times)
- `-m, --remark <REMARK>` - Remarks (can be used multiple times)

### list

List all domains.

```bash
i-rs-domain list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get domain details.

```bash
i-rs-domain get <NAME> [OPTIONS]
```

Options:
- `-s, --show-password` - Show password from keychain (default: hidden)

### update

Update a domain.

```bash
i-rs-domain update <NAME> [OPTIONS]
```

Options:
- `-e, --expiry-date <DATE>` - New expiry date (YYYY-MM-DD)
- `-r, --registrar <REGISTRAR>` - New registrar
- `-p, --password <PASSWORD>` - New password (stored securely in keychain)
- `-t, --tag <TAG>` - New tags
- `-m, --remark <REMARK>` - New remarks

### delete

Delete a domain.

```bash
i-rs-domain delete <NAME>
```

## Examples

```bash
# Add a domain
i-rs-domain add example.com 2025-12-31 --registrar GoDaddy --tag important --remark "Primary domain"

# Add a domain without password
i-rs-domain add github.io 2026-06-15 --tag personal

# List all domains
i-rs-domain list

# Get domain details
i-rs-domain get example.com

# Get domain with password visible
i-rs-domain get example.com --show-password

# Update expiry date
i-rs-domain update example.com --expiry-date 2026-12-31

# Delete a domain
i-rs-domain delete example.com
```

## Data Storage

- macOS: `~/.config/i-rs/domains.json`
- Linux: `~/.config/i-rs/domains.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

**Passwords are NEVER stored in the config file. They go to the OS keychain.**
