# Test Records

Use these test records to verify the installation.

## Test Run Records

### Test 1: Simple Run
```bash
i-rs-run add 2025-06-14 5.0 30
```

Expected:
- Distance: 5.00 km
- Duration: 30:00
- Pace: 6:00/km

### Test 2: Long Run
```bash
i-rs-run add 2025-06-15 10.0 60 -r 145 -w sunny -t long-run
```

Expected:
- Distance: 10.00 km
- Duration: 1:00:00
- Pace: 6:00/km
- Heart Rate: 145 bpm

### Test 3: Interval Training
```bash
i-rs-run add 2025-06-16 8.0 50 -r 160 -t interval --remark "8x400m"
```

Expected:
- Distance: 8.00 km
- Duration: 50:00
- Pace: 6:15/km

## Test Plans

### Test Plan
```bash
i-rs-run plan-add "5K Training Plan" 5.0 6:00 --schedule 1 3 5 -t beginner
```

Expected:
- Name: 5K Training Plan
- Target: 5.00 km
- Pace: 6:00/km

## Test Commands

### List Records
```bash
i-rs-run list
```

### View Statistics
```bash
i-rs-run stats
```

### List Plans
```bash
i-rs-run plan-list
```

## Cleanup

To clean up test data:
```bash
# Get IDs from list command
i-rs-run list

# Delete individual records
i-rs-run delete <ID>

# Delete plans
i-rs-run plan-delete <ID>
```
