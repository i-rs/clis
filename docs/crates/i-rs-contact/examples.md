# i-rs-contact Examples

## Basic Usage

### Adding Contacts

```bash
# Add a simple contact
i-rs-contact add John --phone 13800138000

# Add with email and relationship
i-rs-contact add John --phone 13800138000 --email john@example.com --relationship friend

# Add with tags
i-rs-contact add Alice --phone 13900139000 --tag family --tag important

# Add with remarks
i-rs-contact add Bob --phone 13700137000 --relationship colleague --remark "Met at conference 2024"
```

### Viewing Contacts

```bash
# List all contacts
i-rs-contact list

# Filter by tag
i-rs-contact list --tag family

# Get detailed info
i-rs-contact get John

# Get JSON output
i-rs-contact list --json
```

### Updating Contacts

```bash
# Update phone number
i-rs-contact update John --phone 13800138001

# Update relationship
i-rs-contact update John --relationship colleague

# Add tags
i-rs-contact update Alice --tag work --tag important

# Update multiple fields
i-rs-contact update Bob --phone 13700137001 --email bob@example.com --relationship friend
```

### Deleting Contacts

```bash
# Delete a contact
i-rs-contact delete old_contact
```

## Statistics

```bash
# View all statistics
i-rs-contact stats

# Get JSON output for scripts
i-rs-contact stats --json
```

## Reminders

```bash
# Default reminder (30 days)
i-rs-contact remind

# Custom days threshold
i-rs-contact remind --days 7
i-rs-contact remind --days 60
```

## Real-world Scenarios

### Work Network

```bash
# Add colleagues
i-rs-contact add Alice --phone 13900139000 --email alice@company.com --relationship colleague --tag work
i-rs-contact add Bob --phone 13800138000 --email bob@company.com --relationship colleague --tag work

# View work contacts
i-rs-contact list --tag work
```

### Family

```bash
# Add family members
i-rs-contact add Mom --phone 13700137000 --relationship family --tag family
i-rs-contact add Dad --phone 13600136000 --relationship family --tag family
i-rs-contact add Sister --phone 13500135000 --relationship family --tag family

# View family contacts
i-rs-contact list --tag family
```

### Clients

```bash
# Add clients
i-rs-contact add ClientA --phone 13400134000 --email client@example.com --relationship client --tag business
i-rs-contact add ClientB --phone 13300133000 --email client2@example.com --relationship client --tag business

# View all business contacts
i-rs-contact list --tag business
```

## Output Examples

### List Output

```
╭──────────┬──────────────┬─────────────────┬───────────────┬──────────────┬─────────┬──────────────╮
│ NAME    │ PHONE        │ EMAIL           │ RELATIONSHIP │ LAST CONTACT │ TAGS    │ UPDATED      │
├──────────┼──────────────┼─────────────────┼───────────────┼──────────────┼─────────┼──────────────┤
│ John    │ 13800138000  │ john@example.com│ friend       │ 2024-01-10   │ -       │ 2024-01-10   │
│ Alice   │ 13900139000  │ alice@example.com│ family       │ 2024-01-05   │ family  │ 2024-01-05   │
│ Bob     │ 13700137000  │ bob@example.com │ colleague    │ -            │ work    │ 2024-01-08   │
╰──────────┴──────────────┴─────────────────┴───────────────┴──────────────┴─────────┴──────────────╯

Total: 3 contacts
```

### Stats Output

```
Contact Statistics

Total Contacts: 10

By Relationship:
  friend: 4
  family: 3
  colleague: 2
  client: 1

By Tag:
  work: 5
  family: 3
  important: 2

Needs Reminder (30 days+): 3
  - Bob
  - Carol
  - David
```

### Remind Output

```
Contacts to Reach Out (30 days+)

John (65 days)
  Phone: 13800138000
  Relationship: friend

Alice (45 days)
  Email: alice@example.com
  Relationship: family

Total: 2 contacts need attention
```
