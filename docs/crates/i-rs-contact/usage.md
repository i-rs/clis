# i-rs-contact Usage

## Commands

### add

Create a new contact.

```bash
i-rs-contact add <NAME> [OPTIONS]
```

Arguments:
- `NAME` - Contact name (required)

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

Arguments:
- `NAME` - Contact name (required)

### update

Update a contact.

```bash
i-rs-contact update <NAME> [OPTIONS]
```

Options:
- `-p, --phone <PHONE>` - Update phone number
- `-e, --email <EMAIL>` - Update email address
- `-r, --relationship <REL>` - Update relationship
- `-t, --tag <TAG>` - Update tags
- `-m, --remark <REMARK>` - Update remarks

### delete

Delete a contact.

```bash
i-rs-contact delete <NAME>
```

Arguments:
- `NAME` - Contact name (required)

### stats

View contact statistics.

```bash
i-rs-contact stats
```

Displays:
- Total contact count
- Contacts grouped by relationship
- Contacts grouped by tag
- Contacts needing attention (30+ days without contact)

### remind

Show contacts that need attention.

```bash
i-rs-contact remind [OPTIONS]
```

Options:
- `-d, --days <DAYS>` - Days threshold (default: 30)

### example

Show usage examples.

```bash
i-rs-contact example
```

### skill

Show AI skill documentation.

```bash
i-rs-contact skill [subcommand]
```

Subcommands:
- `summary` - Show skill summary
- `content` - Show skill content
- `raw` - Show raw skill document

## Relationship Options

| Option | Description |
|--------|-------------|
| family | Family members |
| friend | Friends |
| colleague | Work colleagues |
| client | Clients |
| other | Other relationships |

## Data Storage

- macOS: `~/.config/i-rs/contacts.json`
- Linux: `~/.config/i-rs/contacts.json`
- Windows: `~\AppData\Roaming\i-rs\contacts.json`

## Global Flags

- `--json` - Output in JSON format
