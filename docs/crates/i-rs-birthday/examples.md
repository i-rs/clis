# i-rs-birthday Examples

## Basic Usage

### Adding Birthdays

```bash
# Add a friend with birth year
i-rs-birthday add John 06-15 --year 1990 --relationship friend --tag personal

# Add family member
i-rs-birthday add Mom 08-20 --year 1965 --relationship family --tag important

# Add colleague
i-rs-birthday add Colleague 03-10 --relationship colleague --tag work

# Add without birth year (age won't be calculated)
i-rs-birthday add Neighbor 12-25 --relationship neighbor

# Add with multiple tags
i-rs-birthday add BestFriend 04-01 --year 1992 --tag personal --tag important --tag friend

# Add with remarks
i-rs-birthday add Boss 11-15 --year 1975 --relationship work --remark "Likes cake"
```

### Managing Birthdays

```bash
# List all birthdays
i-rs-birthday list

# List family birthdays
i-rs-birthday list --tag family

# List personal birthdays
i-rs-birthday list --tag personal

# Get birthday details
i-rs-birthday get John

# Get Mom's details
i-rs-birthday get Mom
```

### Updating Birthdays

```bash
# Update birth date
i-rs-birthday update John --birth-date 06-20

# Update birth year
i-rs-birthday update John --year 1991

# Update relationship
i-rs-birthday update Mom --relationship "close family"

# Update tags
i-rs-birthday update John --tag work --tag colleague

# Full update
i-rs-birthday update John -d 06-20 -y 1991 -r friend -t personal
```

### Deleting Birthdays

```bash
i-rs-birthday delete John
i-rs-birthday delete OldColleague
```

## Viewing Statistics and Upcoming

### Statistics

```bash
# View overall statistics
i-rs-birthday stats

# Get statistics in JSON format
i-rs-birthday stats --json
```

The stats command shows:
- Total birthdays tracked
- This month's birthdays
- Today's birthdays
- Upcoming in 7 days
- Upcoming in 30 days
- Average age
- Breakdown by relationship

### Upcoming Birthdays

```bash
# View upcoming in next 30 days (default)
i-rs-birthday upcoming

# View upcoming in next 7 days
i-rs-birthday upcoming --days 7

# View upcoming in next 60 days
i-rs-birthday upcoming --days 60

# View upcoming in next 90 days
i-rs-birthday upcoming --days 90

# Get upcoming in JSON format
i-rs-birthday upcoming --json
```

## Organized Birthday Management

### By Relationship

```bash
# Family members
i-rs-birthday add Dad 05-10 --year 1960 --relationship family --tag family
i-rs-birthday add Mom 08-20 --year 1965 --relationship family --tag family
i-rs-birthday add Sister 02-14 --year 1990 --relationship family --tag family

# Friends
i-rs-birthday add John 06-15 --year 1990 --relationship friend --tag personal
i-rs-birthday add Jane 11-20 --year 1988 --relationship friend --tag personal

# Work
i-rs-birthday add Boss 03-01 --year 1975 --relationship colleague --tag work
i-rs-birthday add Colleague1 07-25 --year 1985 --relationship colleague --tag work

# View by relationship
i-rs-birthday list --tag family
i-rs-birthday list --tag work
```

### By Priority

```bash
# Important (don't miss)
i-rs-birthday add Spouse 02-14 --year 1990 --relationship family --tag important --tag priority
i-rs-birthday add Mom 08-20 --year 1965 --relationship family --tag important

# Regular
i-rs-birthday add Friend1 06-15 --year 1992 --relationship friend --tag regular
i-rs-birthday add Friend2 12-25 --year 1988 --relationship friend --tag regular
```

## Real-World Scenarios

### Never Miss a Birthday

```bash
# Set up weekly reminder check
i-rs-birthday upcoming --days 7

# Set up monthly reminder
i-rs-birthday upcoming --days 30

# Get statistics for planning
i-rs-birthday stats
```

### Family Birthday Calendar

```bash
# Add all family members
i-rs-birthday add Dad 05-10 --year 1960 --relationship family --tag family --tag parent
i-rs-birthday add Mom 08-20 --year 1965 --relationship family --tag family --tag parent
i-rs-birthday add Sister 02-14 --year 1990 --relationship family --tag family --tag sibling
i-rs-birthday add Brother 06-05 --year 1993 --relationship family --tag family --tag sibling
i-rs-birthday add Grandmother 12-01 --year 1940 --relationship family --tag family --tag grandparent

# Check upcoming family birthdays
i-rs-birthday upcoming --days 30
```

### Work Birthday Tracking

```bash
# Add colleagues
i-rs-birthday add Colleague1 01-15 --year 1985 --relationship colleague --tag work
i-rs-birthday add Colleague2 03-20 --year 1990 --relationship colleague --tag work
i-rs-birthday add Manager 07-01 --year 1978 --relationship manager --tag work --tag important

# View work birthdays
i-rs-birthday list --tag work

# Check upcoming in next month
i-rs-birthday upcoming --days 30
```

### Complete Setup Example

```bash
# Step 1: Add some birthdays
i-rs-birthday add John 06-15 --year 1990 --relationship friend --tag personal --tag important
i-rs-birthday add Mom 08-20 --year 1965 --relationship family --tag family --tag important
i-rs-birthday add Dad 05-10 --year 1960 --relationship family --tag family
i-rs-birthday add Jane 11-20 --year 1988 --relationship friend --tag personal
i-rs-birthday add Boss 03-01 --year 1975 --relationship colleague --tag work

# Step 2: View all birthdays
i-rs-birthday list

# Step 3: Check upcoming birthdays
i-rs-birthday upcoming

# Step 4: View statistics
i-rs-birthday stats

# Step 5: Get details of a specific person
i-rs-birthday get John

# Step 6: Use JSON output for automation
i-rs-birthday upcoming --days 7 --json > upcoming_birthdays.json
```

## JSON Output

All commands support JSON output for integration with other tools:

```bash
# List in JSON
i-rs-birthday list --json

# Get specific birthday in JSON
i-rs-birthday get John --json

# Stats in JSON
i-rs-birthday stats --json

# Upcoming in JSON
i-rs-birthday upcoming --days 7 --json
```

## Tips and Tricks

### Birthday Format

- Use MM-DD format for birth dates (e.g., 06-15 for June 15th)
- Always include birth year if known for accurate age calculation
- Use tags to organize by category (family, work, personal, etc.)

### Regular Maintenance

```bash
# Weekly: Check upcoming birthdays
i-rs-birthday upcoming --days 7

# Monthly: Review statistics
i-rs-birthday stats

# Quarterly: Full review
i-rs-birthday list
i-rs-birthday upcoming --days 90
```

### Bulk Operations

```bash
# View all birthdays with age
i-rs-birthday list

# Filter to find specific people
i-rs-birthday list --tag family

# Get JSON for external tools
i-rs-birthday list --json > birthdays_backup.json
```
