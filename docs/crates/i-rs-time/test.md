# i-rs-time Test Records

## Test Scenarios

### 1. Start and Stop Timer

```bash
# Start timer
i-rs-time start "Test task 1"

# Wait a few seconds, then stop
i-rs-time stop
```

Expected: Timer starts and records duration correctly.

### 2. Multiple Entries

```bash
i-rs-time start "Task A" --tag work
# stop
i-rs-time start "Task B" --tag personal
# stop
i-rs-time list
```

Expected: Both entries appear in the list.

### 3. Statistics

```bash
i-rs-time stats today
i-rs-time stats week
```

Expected: Shows total time and entry count.

### 4. Reports

```bash
i-rs-time report --days 7
```

Expected: Shows breakdown by day.

### 5. Tag Filtering

```bash
i-rs-time start "Work task" --tag work
# stop
i-rs-time start "Personal task" --tag personal
# stop
i-rs-time list --tag work
```

Expected: Only shows work-tagged entries.

### 6. JSON Output

```bash
i-rs-time list --json
i-rs-time stats today --json
```

Expected: Valid JSON response with proper structure.
