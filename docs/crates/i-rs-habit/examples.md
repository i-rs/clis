# i-rs-habit Examples

## Basic Usage

### Creating Habits

```bash
# Create a simple daily habit
i-rs-habit add daily_walk

# Create with description and tags
i-rs-habit add meditation --description "Morning meditation 10 mins" --frequency daily --tag health

# Create a weekly habit
i-rs-habit add gym --description "Workout at gym" --frequency weekly --tag fitness
```

### Checkin

```bash
# Checkin for today
i-rs-habit checkin daily_walk
i-rs-habit checkin meditation

# Checkin multiple habits
i-rs-habit checkin daily_walk
i-rs-habit checkin reading
```

### Viewing Habits

```bash
# List all habits with streaks
i-rs-habit list

# Filter by tag
i-rs-habit list --tag health

# Get detailed info
i-rs-habit get daily_walk

# Get JSON output
i-rs-habit list --json
```

### Updating Habits

```bash
# Update description
i-rs-habit update daily_walk --description "Walk 45 minutes"

# Change frequency
i-rs-habit update gym --frequency biweekly

# Add tags
i-rs-habit update reading --tag books --tag learning
```

### Deleting Habits

```bash
# Delete a habit
i-rs-habit delete old_habit
```

## Real-world Scenarios

### Morning Routine

```bash
# Create morning habits
i-rs-habit add morning_stretch --description "10 minute stretch" --frequency daily --tag morning
i-rs-habit add drink_water --description "Drink a glass of water" --frequency daily --tag morning
i-rs-habit add breakfast --description "Eat healthy breakfast" --frequency daily --tag morning

# Checkin in the morning
i-rs-habit checkin morning_stretch
i-rs-habit checkin drink_water
i-rs-habit checkin breakfast
```

### Fitness Goals

```bash
# Create fitness habits
i-rs-habit add run --description "Jog 5km" --frequency weekly --tag fitness
i-rs-habit add yoga --description "Yoga session" --frequency weekly --tag fitness
i-rs-habit add pushups --description "20 pushups" --frequency daily --tag fitness

# Track progress
i-rs-habit list --tag fitness
```

## Output Examples

### List Output

```
╭────────────────────┬────────────────────────┬──────────┬────────┬─────────┬──────────────────╮
│ NAME              │ DESCRIPTION            │ FREQUENCY│ STREAK │ TAGS    │ UPDATED          │
├────────────────────┼────────────────────────┼──────────┼────────┼─────────┼──────────────────┤
│ daily_walk        │ Walk 30 minutes        │ daily    │ 7      │ health  │ 2024-01-15 08:30 │
│ meditation        │ Morning meditation     │ daily    │ 14     │ health  │ 2024-01-15 06:00 │
│ gym               │ Workout at gym         │ weekly   │ 3      │ fitness │ 2024-01-14 18:00 │
╰────────────────────┴────────────────────────┴──────────┴────────┴─────────┴──────────────────╯

Total: 3 habits
```