# i-rs-time Examples

## Basic Usage

### Start a Timer

```bash
# Simple timer
i-rs-time start "Working on project"

# With tags
i-rs-time start "Team meeting" --tag meeting

# With tags and remarks
i-rs-time start "Feature implementation" --tag development --remark "User authentication"
```

### Stop a Timer

```bash
i-rs-time stop
```

## Statistics

### View Daily Stats

```bash
# Today's statistics
i-rs-time stats today

# Yesterday's statistics
i-rs-time stats yesterday
```

### View Weekly Stats

```bash
i-rs-time stats week
```

## Reports

### Weekly Report

```bash
i-rs-time report --days 7
```

### Monthly Report

```bash
i-rs-time report --days 30
```

### Custom Date Range

```bash
i-rs-time report --start 2024-01-01 --end 2024-01-31
```

## Managing Entries

### List All Entries

```bash
# All entries
i-rs-time list

# Filter by tag
i-rs-time list --tag work
```

### Get Entry Details

```bash
i-rs-time get abc12345
```

### Delete Entry

```bash
i-rs-time delete abc12345
```

## JSON Output

```bash
# List in JSON
i-rs-time list --json

# Stats in JSON
i-rs-time stats today --json

# Report in JSON
i-rs-time report --days 7 --json
```
