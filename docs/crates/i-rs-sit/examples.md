# i-rs-sit Examples

## Basic Usage

### Recording Sitting Duration

```bash
# Quick recording
i-rs-sit add 60
i-rs-sit add 90
i-rs-sit add 120

# With tags
i-rs-sit add 60 --tag work
i-rs-sit add 120 --tag office
i-rs-sit add 30 --tag home
```

### With Remarks

```bash
# With remarks
i-rs-sit add 60 --remark after lunch
i-rs-sit add 90 --remark during meeting
i-rs-sit add 120 --remark while coding
```

## Viewing Records

```bash
# List all records
i-rs-sit list

# Filter by tag
i-rs-sit list --tag work
i-rs-sit list --tag home
```

## Managing Records

```bash
# Get record details
i-rs-sit get abc12345

# Delete a record
i-rs-sit delete abc12345
```

## Health Tracking

```bash
# Office work day
i-rs-sit add 120 --tag office --remark morning
i-rs-sit add 60 --tag office --remark after lunch
i-rs-sit add 90 --tag office --remark afternoon

# Working from home
i-rs-sit add 60 --tag home --remark morning standup
i-rs-sit add 180 --tag home --remark coding session
```

## Integration Examples

### Stand Up Reminder

```bash
#!/bin/bash
# stand-up.sh - Remind to track sitting

echo "How long have you been sitting?"
read -p "Duration (minutes): " duration

i-rs-sit add $duration

echo "Tracked ${duration} minutes of sitting."
echo "Remember to stand up and move!"
```