---
name: "i-rs-contact"
description: "Manages contacts with relationship tracking and reminders. Invoke when user wants to add, list, or track contacts."
---

# i-rs-contact

Contact management CLI tool for managing contacts, tracking relationships and contact history.

## Storage

- Config: `~/.config/i-rs/contacts.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Create a new contact.

```bash
i-rs-contact add <NAME> [OPTIONS]
```

Options:
- `-p, --phone <PHONE>` - Phone number
- `-e, --email <EMAIL>` - Email address
- `-r, --relationship <REL>` - Relationship (family/friend/colleague/client/other)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-m, --remark <REMARK>` - Remarks (can be repeated)

### list

List all contacts.

```bash
i-rs-contact list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get contact details.

```bash
i-rs-contact get <NAME>
```

### update

Update a contact.

```bash
i-rs-contact update <NAME> [OPTIONS]
```

### delete

Delete a contact.

```bash
i-rs-contact delete <NAME>
```

### stats

View contact statistics.

```bash
i-rs-contact stats
```

Displays:
- Total contact count
- Contacts by relationship
- Contacts by tag
- Contacts needing reminder (30+ days)

### remind

Show contacts that need attention.

```bash
i-rs-contact remind [OPTIONS]
```

Options:
- `-d, --days <DAYS>` - Days threshold (default: 30)

### data

Manage data (export, import, clear).

```bash
i-rs-contact data export
i-rs-contact data import [FILE]
i-rs-contact data clear
```

### example

Show usage examples.

```bash
i-rs-contact example
```

### skill

Show skill information.

```bash
i-rs-contact skill [summary|content|raw]
```

## Examples

```bash
# Add a contact
i-rs-contact add John --phone 13800138000 --email john@example.com --relationship friend

# Add with tags
i-rs-contact add Alice --phone 13900139000 --tag family --tag important

# List contacts
i-rs-contact list

# View statistics
i-rs-contact stats

# Remind to reach out
i-rs-contact remind
```

## Relationships

| Option | Description |
|--------|-------------|
| family | Family members |
| friend | Friends |
| colleague | Work colleagues |
| client | Clients |
| other | Other |

## Contact Tracking

The tool tracks:
- Last contact date
- Contact count
- Days since last contact

Use `remind` to identify contacts you haven't reached out to recently.
