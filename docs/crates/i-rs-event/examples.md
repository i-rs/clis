# i-rs-event Examples

## Adding Events

### Meeting

```bash
# Basic meeting
i-rs-event add "Team Standup" --date 2024-03-15 --type meeting --location "Zoom"

# Meeting with participants and tags
i-rs-event add "Sprint Planning" --date 2024-03-15 10:00 --type meeting \
  --location "Conference Room A" \
  -p "Alice,Bob,Charlie" \
  -t work,sprint,important
```

### Gathering

```bash
# Birthday party
i-rs-event add "Birthday Party" --date 2024-04-20 --type gathering \
  --location "Home" \
  -p "Mom,Dad,Sister" \
  -t personal,celebration

# Holiday gathering
i-rs-event add "New Year Party" --date 2024-12-31 --type gathering \
  --location "Downtown Club" \
  -p "Friends,Colleagues" \
  -t holiday,party
```

### Course

```bash
# Workshop
i-rs-event add "Rust Workshop" --date 2024-05-10 --type course \
  --location "Online" \
  -t learning,tech,rust

# Training
i-rs-event add "Leadership Training" --date 2024-06-01 --type course \
  --location "Training Center" \
  -p "Managers,Team Leads" \
  -t work,training,leadership
```

### Other Events

```bash
# Conference
i-rs-event add "Tech Conference 2024" --date 2024-09-15 --type other \
  --location "Convention Center" \
  -t tech,networking,learning

# With remarks
i-rs-event add "Doctor Appointment" --date 2024-03-20 --type other \
  --location "City Hospital" \
  -t health \
  -r "Bring insurance card" \
  -r "Arrive 15 minutes early"
```

## Listing Events

```bash
# List all events
i-rs-event list

# Filter by tag
i-rs-event list --tag work

# Filter by event type
i-rs-event list --type meeting

# Combined filters
i-rs-event list --tag important --type meeting
```

## Getting Event Details

```bash
# Basic get
i-rs-event get "Team Standup"

# JSON output
i-rs-event get "Team Standup" --json
```

## Deleting Events

```bash
# Delete an event
i-rs-event delete "Old Meeting"

# Confirm deletion
i-rs-event delete "Old Meeting" --json
```

## Statistics

```bash
# Current year statistics
i-rs-event stats

# Specific year
i-rs-event stats --year 2024

# JSON output
i-rs-event stats --json
```

## JSON Output

```bash
# List as JSON
i-rs-event list --json

# Get as JSON
i-rs-event get "Team Standup" --json

# Stats as JSON
i-rs-event stats --year 2024 --json
```

## Using Tags Effectively

```bash
# Add multiple tags
i-rs-event add "Event" --date 2024-03-15 -t "work,urgent,team"

# Filter by any tag
i-rs-event list --tag urgent

# List all work events
i-rs-event list --tag work
```
