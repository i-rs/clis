# i-rs-birthday

Birthday reminder CLI tool for managing birthdays and never missing an important date.

## Features

- Add, update, delete, and list birthdays
- Track relationships (family, friend, colleague, etc.)
- Automatic age calculation
- Days until birthday countdown
- Upcoming birthdays view
- Statistics and insights
- Tag support for organization
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-birthday
# or
brew install i-rs/homebrew-tap/i-rs-birthday
```

## Quick Start

```bash
# Add a birthday
i-rs-birthday add John 06-15 --year 1990 --relationship friend --tag personal

# Add family member
i-rs-birthday add Mom 08-20 --year 1965 --relationship family --tag important

# List all birthdays
i-rs-birthday list

# List by tag
i-rs-birthday list --tag family

# Get birthday details
i-rs-birthday get John

# View upcoming birthdays
i-rs-birthday upcoming

# View statistics
i-rs-birthday stats
```

## Usage

### add

Add a new birthday.

```bash
i-rs-birthday add <NAME> <BIRTH_DATE> [OPTIONS]
```

Arguments:
- `NAME` - Person's name (required)
- `BIRTH_DATE` - Birthday (MM-DD format, required)

Options:
- `-y, --year <YEAR>` - Birth year (optional, for age calculation)
- `-r, --relationship <RELATIONSHIP>` - Relationship type (optional)
- `-t, --tag <TAG>` - Tags (can be specified multiple times)
- `-m, --remark <REMARK>` - Remarks (can be specified multiple times)

### list

List all birthdays, optionally filtered by tag.

```bash
i-rs-birthday list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get detailed information about a birthday.

```bash
i-rs-birthday get <NAME>
```

### update

Update an existing birthday.

```bash
i-rs-birthday update <NAME> [OPTIONS]
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
i-rs-birthday upcoming [OPTIONS]
```

Options:
- `-d, --days <DAYS>` - Number of days to look ahead (default: 30)

## Examples

```bash
# Add a friend
i-rs-birthday add John 06-15 --year 1990 --relationship friend --tag personal

# Add family member
i-rs-birthday add Mom 08-20 --year 1965 --relationship family --tag important

# Add colleague
i-rs-birthday add Colleague 03-10 --relationship colleague --tag work

# List all birthdays
i-rs-birthday list

# List family birthdays
i-rs-birthday list --tag family

# Get details
i-rs-birthday get John

# Update birthday
i-rs-birthday update John --birth-date 06-20 --tag work

# Delete
i-rs-birthday delete John

# View upcoming (next 7 days)
i-rs-birthday upcoming --days 7

# View statistics
i-rs-birthday stats

# JSON output
i-rs-birthday list --json
i-rs-birthday stats --json
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/birthdays.json`
- Linux: `~/.config/i-rs/birthdays.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-birthday list
```

## License

MIT OR Apache-2.0
