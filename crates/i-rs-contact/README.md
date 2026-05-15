# i-rs-contact

Contact management CLI tool for managing contacts, tracking relationships, and contact history.

## Features

- Contact information (name, phone, email, relationship)
- Tag-based categorization (friend, colleague, family, etc.)
- Recent contact tracking
- Contact frequency statistics
- Reminder for contacts not reached in 30+ days

## Install

```bash
cargo build -p i-rs-contact
# or
cargo install --path crates/i-rs-contact
```

## Quick Start

```bash
# Add a contact
i-rs-contact add John --phone 13800138000 --email john@example.com --relationship friend

# Add contact with tags
i-rs-contact add Alice --phone 13900139000 --tag family --tag important

# List all contacts
i-rs-contact list

# Filter by tag
i-rs-contact list --tag family

# View contact details
i-rs-contact get John

# Update contact
i-rs-contact update John --phone 13800138001

# Delete contact
i-rs-contact delete old_contact

# View statistics
i-rs-contact stats

# Remind to contact
i-rs-contact remind

# Custom reminder days
i-rs-contact remind --days 7
```

## Data Storage

- macOS: `~/.config/i-rs/contacts.json`
- Linux: `~/.config/i-rs/contacts.json`
- Windows: `~\AppData\Roaming\i-rs\contacts.json`

## License

MIT OR Apache-2.0
