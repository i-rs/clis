---
name: "i-rs-birthday"
description: "Manages birthdays (add/list/get/update/delete/stats/upcoming). Invoke when user needs to track birthdays, calculate ages, or view upcoming birthdays."
---

# i-rs-birthday

Birthday reminder CLI tool for managing birthdays and never missing an important date.

## Storage

- Config: `~/.config/i-rs/birthdays.json`

## Commands

### add

Add a new birthday.

```bash
i-rs-birthday add <NAME> <BIRTH_DATE>
```

Arguments:
- `NAME` - Person's name (required)
- `BIRTH_DATE` - Birthday in MM-DD format (required)

Options:
- `-y, --year <YEAR>` - Birth year (optional, for age calculation)
- `-r, --relationship <RELATIONSHIP>` - Relationship type (optional)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-m, --remark <REMARK>` - Remarks (can be repeated)

### list

List all birthdays.

```bash
i-rs-birthday list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get birthday details.

```bash
i-rs-birthday get <NAME>
```

### update

Update a birthday.

```bash
i-rs-birthday update <NAME>
```

Options:
- `-d, --birth-date <DATE>` - New birthday (MM-DD)
- `-y, --year <YEAR>` - New birth year
- `-r, --relationship <RELATIONSHIP>` - New relationship
- `-t, --tag <TAG>` - New tags
- `-m, --remark <REMARK>` - New remarks

### delete

Delete a birthday.

```bash
i-rs-birthday delete <NAME>
```

### stats

View birthday statistics.

```bash
i-rs-birthday stats
```

Shows:
- Total birthdays
- This month's birthdays
- Today's birthdays
- Upcoming in 7/30 days
- Average age
- Breakdown by relationship

### upcoming

View upcoming birthdays.

```bash
i-rs-birthday upcoming
```

Options:
- `-d, --days <DAYS>` - Number of days to look ahead (default: 30)

## Examples

```bash
# Add a friend
i-rs-birthday add John 06-15 --year 1990 --relationship friend --tag personal

# Add family member
i-rs-birthday add Mom 08-20 --year 1965 --relationship family --tag important

# List all birthdays
i-rs-birthday list

# List by tag
i-rs-birthday list --tag family

# Get details
i-rs-birthday get John

# View upcoming
i-rs-birthday upcoming

# View upcoming in 7 days
i-rs-birthday upcoming --days 7

# View statistics
i-rs-birthday stats

# Update birthday
i-rs-birthday update John --birth-date 06-20

# Delete
i-rs-birthday delete John

# JSON output
i-rs-birthday list --json
i-rs-birthday stats --json
```
