# i-rs-birthday Usage Guide

## Commands Overview

| Command | Description |
|---------|-------------|
| `add` | Add a new birthday |
| `list` | List all birthdays |
| `get` | Get birthday details |
| `update` | Update a birthday |
| `delete` | Delete a birthday |
| `stats` | View statistics |
| `upcoming` | View upcoming birthdays |

---

## add

Add a new birthday.

```bash
i-rs-birthday add <NAME> <BIRTH_DATE> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Person's name | Yes |
| `BIRTH_DATE` | Birthday in MM-DD format | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-y` | `--year` | Birth year (for age calculation) |
| `-r` | `--relationship` | Relationship type |
| `-t` | `--tag` | Tags (can be repeated) |
| `-m` | `--remark` | Remarks (can be repeated) |

### Examples

```bash
i-rs-birthday add John 06-15 --year 1990 --relationship friend
i-rs-birthday add Mom 08-20 --year 1965 --relationship family --tag important
```

---

## list

List all birthdays, optionally filtered by tag.

```bash
i-rs-birthday list [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-t` | `--tag` | Filter by tag |

### Examples

```bash
i-rs-birthday list
i-rs-birthday list --tag family
i-rs-birthday list --tag work
```

---

## get

Get detailed information about a birthday.

```bash
i-rs-birthday get <NAME>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Person's name | Yes |

### Examples

```bash
i-rs-birthday get John
i-rs-birthday get Mom
```

---

## update

Update an existing birthday.

```bash
i-rs-birthday update <NAME> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Person's name | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-d` | `--birth-date` | New birthday (MM-DD) |
| `-y` | `--year` | New birth year |
| `-r` | `--relationship` | New relationship |
| `-t` | `--tag` | New tags |
| `-m` | `--remark` | New remarks |

### Examples

```bash
i-rs-birthday update John --birth-date 06-20
i-rs-birthday update John --year 1991
i-rs-birthday update Mom --relationship "close family"
```

---

## delete

Delete a birthday.

```bash
i-rs-birthday delete <NAME>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Person's name | Yes |

### Examples

```bash
i-rs-birthday delete John
i-rs-birthday delete Mom
```

---

## stats

View birthday statistics including totals, averages, and breakdowns.

```bash
i-rs-birthday stats
```

### Output Includes

- Total birthdays
- This month's birthdays
- Today's birthdays
- Upcoming in 7 days
- Upcoming in 30 days
- Average age (if birth years known)
- Breakdown by relationship

### Examples

```bash
i-rs-birthday stats
i-rs-birthday stats --json
```

---

## upcoming

View upcoming birthdays within a specified number of days.

```bash
i-rs-birthday upcoming [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-d` | `--days` | Number of days to look ahead (default: 30) |

### Examples

```bash
i-rs-birthday upcoming
i-rs-birthday upcoming --days 7
i-rs-birthday upcoming --days 60
i-rs-birthday upcoming --days 90
```

---

## Global Options

All commands support the following global option:

| Short | Long | Description |
|-------|------|-------------|
| | `--json` | Output in JSON format |

### Examples

```bash
i-rs-birthday list --json
i-rs-birthday get John --json
i-rs-birthday stats --json
i-rs-birthday upcoming --json
```
