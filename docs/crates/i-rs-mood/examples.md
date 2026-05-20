# i-rs-mood Examples

## Basic Usage

### Recording Mood

```bash
# Using words
i-rs-mood add good --date 2025-01-15
i-rs-mood add great --date 2025-01-16
i-rs-mood add okay --date 2025-01-17
i-rs-mood add bad --date 2025-01-18
i-rs-mood add terrible --date 2025-01-19

# Using numbers (1-5)
i-rs-mood add 4 --date 2025-01-20
i-rs-mood add 5 --date 2025-01-21
i-rs-mood add 3 --date 2025-01-22
i-rs-mood add 2 --date 2025-01-23
i-rs-mood add 1 --date 2025-01-24

# Using emoji
i-rs-mood add 😊 --date 2025-01-25
i-rs-mood add 🙂 --date 2025-01-26
i-rs-mood add 😐 --date 2025-01-27
i-rs-mood add 😔 --date 2025-01-28
i-rs-mood add 😢 --date 2025-01-29
```

### Viewing Records

```bash
# List all records
i-rs-mood list

# List last 7 days
i-rs-mood list --days 7

# List last 30 days
i-rs-mood list --days 30

# List with calendar view
i-rs-mood list --calendar

# Full output (calendar + last 30 days)
i-rs-mood list --days 30 --calendar
```

### Updating Records

```bash
# Correct a mood entry
i-rs-mood update abc12345 --mood okay

# Update mood and add note
i-rs-mood update abc12345 --mood great --content "Changed my mind"

# Add new tags
i-rs-mood update abc12345 --tag newtag

# Update notes
i-rs-mood update abc12345 --content "Updated note"
```

### Deleting Records

```bash
# Remove incorrect entry
i-rs-mood delete abc12345
```

## With Tags

### Single Tag

```bash
i-rs-mood add good --tag work --date 2025-01-15
i-rs-mood add great --tag weekend --date 2025-01-16
i-rs-mood add bad --tag monday --date 2025-01-17
```

### Multiple Tags

```bash
i-rs-mood add great --tag work --tag achievement --date 2025-01-18
i-rs-mood add good --tag exercise --tag health --date 2025-01-19
i-rs-mood add okay --tag sick --tag health --tag work --date 2025-01-20
```

## With Notes

### Single Note

```bash
i-rs-mood add okay --content "Monday blues" --date 2025-01-15
i-rs-mood add good --content "Project completed" --date 2025-01-16
i-rs-mood add great --content "Got promotion" --date 2025-01-17
```

### Multiple Notes

```bash
i-rs-mood add great --content "Team lunch" --content "Finished project" --content "Good feedback" --date 2025-01-18
i-rs-mood add okay --content "Busy day" --content "Many meetings" --content "No time for deep work" --date 2025-01-19
```

## Combining Tags and Notes

### Work Day Mood

```bash
i-rs-mood add good \
  --tag work \
  --content "Sprint planning went well" \
  --content "Code review completed" \
  --date 2025-01-15
```

### Weekend Mood

```bash
i-rs-mood add great \
  --tag weekend \
  --tag family \
  --content "Picnic with family" \
  --content "Weather was perfect" \
  --date 2025-01-18
```

### Health Day

```bash
i-rs-mood add bad \
  --tag health \
  --tag sick \
  --content "Caught a cold" \
  --content "Need rest" \
  --date 2025-01-20
```

## Calendar View

### Weekly Calendar

```bash
i-rs-mood list --days 7 --calendar
```

Output:
```
 ID       DATE       MOOD     TAGS        CONTENT
 abc12345 2025-01-13 🙂 Good  -          -
 abc23456 2025-01-14 😊 Great work        Finished project
 abc34567 2025-01-15 😐 Okay   monday     Monday blues
 abc45678 2025-01-16 🙂 Good  work        -
 abc56789 2025-01-17 😊 Great weekend      Family time
 abc67890 2025-01-18 🙂 Good  exercise    -
 abc78901 2025-01-19 😐 Okay   sick        -

Total: 7 moods

Statistics:
  Best:      😊 Great
  Worst:     😐 Okay
  Average:   3.6/5

Mood Calendar:
───────────────────────────────────────
 🙂 😊 😐 🙂 😊 🙂 😐

Legend: 😊 Great  🙂 Good  😐 Okay  😔 Bad  😢 Terrible
```

## Advanced Usage

### Daily Mood Journal

```bash
# Create a daily habit
i-rs-mood add good --tag habit --content "First day of tracking" --date 2025-01-01
i-rs-mood add great --tag habit --content "Kept going" --date 2025-01-02
i-rs-mood add good --tag habit --content "Building momentum" --date 2025-01-03
```

### Work-Life Balance Tracking

```bash
# Track how work affects mood
i-rs-mood add bad --tag work --tag deadline --content "Project deadline stress" --date 2025-01-15
i-rs-mood add okay --tag work --content "Meeting heavy day" --date 2025-01-16
i-rs-mood add good --tag work --tag completed --content "Project shipped!" --date 2025-01-17

# Track how rest affects mood
i-rs-mood add great --tag weekend --tag rest --content "Full day of rest" --date 2025-01-18
i-rs-mood add great --tag weekend --tag family --content "Spent time with kids" --date 2025-01-19
```

### Exercise Impact on Mood

```bash
# Track mood on exercise days vs rest days
i-rs-mood add okay --tag rest-day --date 2025-01-15
i-rs-mood add good --tag exercise --tag workout --content "Morning run" --date 2025-01-16
i-rs-mood add great --tag exercise --tag workout --content "Gym session" --date 2025-01-17
```

## Integration Examples

### Daily Mood Reminder Script

```bash
#!/bin/bash
# daily-mood.sh - Prompt for mood at end of day

TODAY=$(date +%Y-%m-%d)

echo "How are you feeling today?"
echo "1: 😊 Great"
echo "2: 🙂 Good"
echo "3: 😐 Okay"
echo "4: 😔 Bad"
echo "5: 😢 Terrible"
read -p "Enter number (1-5): " mood

case $mood in
    1) i-rs-mood add great --date $TODAY ;;
    2) i-rs-mood add good --date $TODAY ;;
    3) i-rs-mood add okay --date $TODAY ;;
    4) i-rs-mood add bad --date $TODAY ;;
    5) i-rs-mood add terrible --date $TODAY ;;
    *) echo "Invalid mood"; exit 1 ;;
esac

echo "Mood logged!"
```

### Weekly Mood Review

```bash
#!/bin/bash
# weekly-mood.sh - Weekly mood summary

echo "=== Mood Summary - Last 7 Days ==="
i-rs-mood list --days 7

echo ""
echo "=== Mood Calendar - Last 7 Days ==="
i-rs-mood list --days 7 --calendar
```
