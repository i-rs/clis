# i-rs-water Examples

## Basic Usage

### Recording Water Intake

```bash
# Quick recording
i-rs-water add 250
i-rs-water add 500
i-rs-water add 1000

# With tags
i-rs-water add 300 --tag morning
i-rs-water add 400 --tag afternoon
i-rs-water add 300 --tag evening

# With remarks
i-rs-water add 250 --remark after workout
i-rs-water add 500 --remark before meal
```

## Viewing Records

```bash
# List all records
i-rs-water list

# Filter by tag
i-rs-water list --tag morning
i-rs-water list --tag workout
```

## Managing Records

```bash
# Get record details
i-rs-water get abc12345

# Delete a record
i-rs-water delete abc12345
```

## Daily Hydration Tracking

```bash
# Morning
i-rs-water add 250 --tag morning --tag wake-up

# Throughout the day
i-rs-water add 300 --tag morning
i-rs-water add 400 --tag afternoon --tag after-lunch
i-rs-water add 300 --tag evening

# Workout days
i-rs-water add 500 --tag workout --remark post-gym
i-rs-water add 300 --tag workout --remark during-workout
```

## Integration Examples

### Daily Reminder Script

```bash
#!/bin/bash
# daily-water.sh - Log water intake

TODAY=$(date +%Y-%m-%d)
echo "How much water did you drink?"
read -p "Amount (ml): " amount

i-rs-water add $amount --tag daily

echo "Logged ${amount}ml!"
```