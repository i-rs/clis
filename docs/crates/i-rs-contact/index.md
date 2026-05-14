# i-rs-contact

Contact management CLI tool for managing contacts, tracking relationships and contact history.

## Overview

i-rs-contact helps you manage your contacts and maintain relationships. It supports:
- Contact information (name, phone, email, relationship)
- Tag-based organization
- Last contact tracking
- Contact frequency statistics
- Reminder for out-of-touch contacts

## Quick Start

```bash
# Add a contact
i-rs-contact add John --phone 13800138000 --email john@example.com --relationship friend

# Add contact with tags
i-rs-contact add Alice --phone 13900139000 --tag family --tag important

# List all contacts
i-rs-contact list

# View contact details
i-rs-contact get John

# Check statistics
i-rs-contact stats

# Remind to reach out
i-rs-contact remind
```

## Key Features

- **Contact Tracking**: Store name, phone, email, and relationship info
- **Tag Organization**: Organize contacts with custom tags
- **Contact History**: Track last contact date and frequency
- **Statistics**: View contact distribution by relationship and tag
- **Reminders**: Identify contacts you haven't reached out to recently
- **JSON Output**: Use `--json` flag for programmatic access

## Data Storage

- macOS: `~/.config/i-rs/contacts.json`
- Linux: `~/.config/i-rs/contacts.json`
- Windows: `~\AppData\Roaming\i-rs\contacts.json`
