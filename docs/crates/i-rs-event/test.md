# i-rs-event Test Records

## Test Setup

```bash
# Build the tool
cargo build -p i-rs-event
```

## Test Commands

### Add Events

```bash
# Test 1: Add a meeting
i-rs-event add "Test Meeting 1" --date 2024-03-15 --type meeting --location "Room A" -p "Alice,Bob" -t test,meeting
# Expected: Success message

# Test 2: Add a gathering
i-rs-event add "Test Gathering 1" --date 2024-04-20 --type gathering --location "Park" -p "Charlie,Dave" -t test,gathering
# Expected: Success message

# Test 3: Add a course
i-rs-event add "Test Course 1" --date 2024-05-10 --type course --location "Online" -t test,course
# Expected: Success message

# Test 4: Add an event with remarks
i-rs-event add "Test Event 1" --date 2024-06-01 --type other -t test -r "Note 1" -r "Note 2"
# Expected: Success message
```

### List Events

```bash
# Test 5: List all events
i-rs-event list
# Expected: Table with 4 events

# Test 6: Filter by tag
i-rs-event list --tag test
# Expected: 4 events

# Test 7: Filter by type
i-rs-event list --type meeting
# Expected: 1 event

# Test 8: List with JSON output
i-rs-event list --json
# Expected: JSON array of events
```

### Get Events

```bash
# Test 9: Get event details
i-rs-event get "Test Meeting 1"
# Expected: Full details including participants

# Test 10: Get non-existent event
i-rs-event get "NonExistent"
# Expected: Error message
```

### Statistics

```bash
# Test 11: View statistics
i-rs-event stats
# Expected: Statistics for current year

# Test 12: Statistics for specific year
i-rs-event stats --year 2024
# Expected: Statistics for 2024

# Test 13: Statistics JSON output
i-rs-event stats --json
# Expected: JSON statistics
```

### Delete Events

```bash
# Test 14: Delete an event
i-rs-event delete "Test Event 1"
# Expected: Success message

# Test 15: Delete non-existent event
i-rs-event delete "NonExistent"
# Expected: Error message

# Test 16: Verify deletion
i-rs-event list
# Expected: 3 events remaining
```

## Cleanup

```bash
# Remove test events
i-rs-event delete "Test Meeting 1"
i-rs-event delete "Test Gathering 1"
i-rs-event delete "Test Course 1"
```
