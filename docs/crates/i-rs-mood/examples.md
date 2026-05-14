# i-rs-mood Examples

## Basic Usage

### Recording Mood

```bash
# Using words
i-rs-mood add 2025-01-15 good
i-rs-mood add 2025-01-16 great
i-rs-mood add 2025-01-17 okay
i-rs-mood add 2025-01-18 bad
i-rs-mood add 2025-01-19 terrible

# Using numbers (1-5)
i-rs-mood add 2025-01-20 4
i-rs-mood add 2025-01-21 5
i-rs-mood add 2025-01-22 3
i-rs-mood add 2025-01-23 2
i-rs-mood add 2025-01-24 1

# Using emoji
i-rs-mood add 2025-01-25 😊
i-rs-mood add 2025-01-26 🙂
i-rs-mood add 2025-01-27 😐
i-rs-mood add 2025-01-28 😔
i-rs-mood add 2025-01-29 😢
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
i-rs-mood update 2025-01-15 --mood okay

# Update mood and add note
i-rs-mood update 2025-01-16 --mood great --content "Changed my mind"

# Add new tags
i-rs-mood update 2025-01-17 --tag newtag

# Update notes
i-rs-mood update 2025-01-18 --content "Updated note"
```

### Deleting Records

```bash
# Remove incorrect entry
i-rs-mood delete 2025-01-15
```

## With Tags

### Single Tag

```bash
i-rs-mood add 2025-01-15 good --tag work
i-rs-mood add 2025-01-16 great --tag weekend
i-rs-mood add 2025-01-17 bad --tag monday
```

### Multiple Tags

```bash
i-rs-mood add 2025-01-18 great --tag work --tag achievement
i-rs-mood add 2025-01-19 good --tag exercise --tag health
i-rs-mood add 2025-01-20 okay --tag sick --tag health --tag work
```

## With Notes

### Single Note

```bash
i-rs-mood add 2025-01-15 okay --content "Monday blues"
i-rs-mood add 2025-01-16 good --content "Project completed"
i-rs-mood add 2025-01-17 great --content "Got promotion"
```

### Multiple Notes

```bash
i-rs-mood add 2025-01-18 great --content "Team lunch" --content "Finished project" --content "Good feedback"
i-rs-mood add 2025-01-19 okay --content "Busy day" --content "Many meetings" --content "No time for deep work"
```

## Combining Tags and Notes

### Work Day Mood

```bash
i-rs-mood add 2025-01-15 good \
  --tag work \
  --content "Sprint planning went well" \
  --content "Code review completed"
```

### Weekend Mood

```bash
i-rs-mood add 2025-01-18 great \
  --tag weekend \
  --tag family \
  --content "Picnic with family" \
  --content "Weather was perfect"
```

### Health Day

```bash
i-rs-mood add 2025-01-20 bad \
  --tag health \
  --tag sick \
  --content "Caught a cold" \
  --content "Need rest"
```

## Calendar View

### Weekly Calendar

```bash
i-rs-mood list --days 7 --calendar
```

Output:
```
 DATE       MOOD     TAGS        CONTENT
 2025-01-13 🙂 Good  -          -
 2025-01-14 😊 Great work        Finished project
 2025-01-15 😐 Okay   monday     Monday blues
 2025-01-16 🙂 Good  work        -
 2025-01-17 😊 Great weekend      Family time
 2025-01-18 🙂 Good  exercise    -
 2025-01-19 😐 Okay   sick        -

Total: 7 records

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
i-rs-mood add 2025-01-01 good --tag habit --content "First day of tracking"
i-rs-mood add 2025-01-02 great --tag habit --content "Kept going"
i-rs-mood add 2025-01-03 good --tag habit --content "Building momentum"
```

### Work-Life Balance Tracking

```bash
# Track how work affects mood
i-rs-mood add 2025-01-15 bad --tag work --tag deadline --content "Project deadline stress"
i-rs-mood add 2025-01-16 okay --tag work --content "Meeting heavy day"
i-rs-mood add 2025-01-17 good --tag work --tag completed --content "Project shipped!"

# Track how rest affects mood
i-rs-mood add 2025-01-18 great --tag weekend --tag rest --content "Full day of rest"
i-rs-mood add 2025-01-19 great --tag weekend --tag family --content "Spent time with kids"
```

### Exercise Impact on Mood

```bash
# Track mood on exercise days vs rest days
i-rs-mood add 2025-01-15 okay --tag rest-day
i-rs-mood add 2025-01-16 good --tag exercise --tag workout --content "Morning run"
i-rs-mood add 2025-01-17 great --tag exercise --tag workout --content "Gym session"
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
    1) i-rs-mood add $TODAY great ;;
    2) i-rs-mood add $TODAY good ;;
    3) i-rs-mood add $TODAY okay ;;
    4) i-rs-mood add $TODAY bad ;;
    5) i-rs-mood add $TODAY terrible ;;
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
