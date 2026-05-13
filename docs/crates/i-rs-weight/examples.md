# i-rs-weight Examples

## Basic Usage

### Recording Weight

```bash
# Daily logging
i-rs-weight add 2025-01-15 75.5
i-rs-weight add 2025-01-16 75.2
i-rs-weight add 2025-01-17 75.0
i-rs-weight add 2025-01-18 74.8

# With notes
i-rs-weight add 2025-01-19 74.5 --remark "Morning weight"
i-rs-weight add 2025-01-20 74.9 --remark "After large meal"
i-rs-weight add 2025-01-21 74.3 --remark "Post workout" --remark "Fasted"
```

### Viewing Records

```bash
# List all records
i-rs-weight list

# List last 7 days
i-rs-weight list --days 7

# List last 30 days
i-rs-weight list --days 30

# List with chart
i-rs-weight list --chart

# List with statistics
i-rs-weight list --stats

# Full output (chart + stats + last 30 days)
i-rs-weight list --days 30 --chart --stats
```

### Updating Records

```bash
# Correct a measurement
i-rs-weight update 2025-01-15 --weight 76.0

# Add note to existing record
i-rs-weight update 2025-01-15 --remark "After breakfast"

# Full update
i-rs-weight update 2025-01-16 -w 75.5 --remark "Corrected"
```

### Deleting Records

```bash
# Remove incorrect entry
i-rs-weight delete 2025-01-15
```

## Advanced Usage

### Weekly Weight Loss Tracking

```bash
# Week 1
i-rs-weight add 2025-01-01 80.0 --remark "Starting weight"
i-rs-weight add 2025-01-02 79.8
i-rs-weight add 2025-01-03 79.5
i-rs-weight add 2025-01-04 79.3
i-rs-weight add 2025-01-05 79.0
i-rs-weight add 2025-01-06 78.8
i-rs-weight add 2025-01-07 78.5

# Week 2
i-rs-weight add 2025-01-08 78.2
i-rs-weight add 2025-01-09 78.0
i-rs-weight add 2025-01-10 77.8
i-rs-weight add 2025-01-11 77.5
i-rs-weight add 2025-01-12 77.3
i-rs-weight add 2025-01-13 77.0
i-rs-weight add 2025-01-14 76.8

# View progress
i-rs-weight list --days 14 --chart --stats
```

### Workout Context Tracking

```bash
# Track weight with workout notes
i-rs-weight add 2025-01-15 70.0 --remark "Rest day"
i-rs-weight add 2025-01-16 70.2 --remark "Upper body workout"
i-rs-weight add 2025-01-17 69.8 --remark "Lower body workout" --remark "Sweated heavily"
i-rs-weight add 2025-01-18 70.1 --remark "Rest day"
i-rs-weight add 2025-01-19 69.9 --remark "Cardio day"
```

### Bulk Update for Data Correction

```bash
# If scale was off by 0.5kg, update multiple entries
i-rs-weight update 2025-01-01 --weight 75.5
i-rs-weight update 2025-01-02 --weight 75.2
i-rs-weight update 2025-01-03 --weight 75.0
```

### Monthly Progress Review

```bash
# View last month's data
i-rs-weight list --days 30 --chart --stats

# Example output:
# Weight Trend (Last 30 days)
# ─────────────────────────────────────
#  75.0 ●
#  74.5    │
#  74.0 ●──│──●──●
#  73.5    │       │
#  73.0 ●─────────●
#  72.5
#
#  01-01               01-15
#
#   Min: 72.5 kg
#   Max: 75.0 kg
#   Avg: 73.9 kg
#   Change: 75.0 → 72.5 (-2.5 kg ↓)
```

## Integration Examples

### Scripting Daily Weight

```bash
#!/bin/bash
# daily-weight.sh - Log daily weight

TODAY=$(date +%Y-%m-%d)
WEIGHT=$1

if [ -z "$WEIGHT" ]; then
    echo "Usage: $0 <weight>"
    exit 1
fi

i-rs-weight add "$TODAY" "$WEIGHT" --remark "Morning weight"
echo "Logged: $TODAY - ${WEIGHT}kg"
```

### Weekly Summary Script

```bash
#!/bin/bash
# weekly-summary.sh

echo "=== Weight Summary - Last 7 Days ==="
i-rs-weight list --days 7 --stats

echo ""
echo "=== Weight Trend - Last 7 Days ==="
i-rs-weight list --days 7 --chart
```

### Monthly Progress Report

```bash
#!/bin/bash
# monthly-report.sh

echo "=== Monthly Weight Report ==="
echo "Date: $(date +%Y-%m)"
echo ""
i-rs-weight list --days 30 --chart --stats
```