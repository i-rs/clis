# i-rs-goal Test Records

## Test Setup

```bash
# Create test goals
i-rs-goal add "Test Goal 1" --target 1000 --deadline 2025-12-31
i-rs-goal add "Test Goal 2" --target 5000 --deadline 2026-06-01 --tags test,priority
i-rs-goal add "Test Goal 3" --target 10000 --deadline 2026-12-31 --milestones "Half:5000,Full:10000"
```

## Test Commands

### List all goals
```bash
i-rs-goal list
```

### Get goal details
```bash
i-rs-goal get "Test Goal 1"
```

### Deposit to goal
```bash
i-rs-goal deposit "Test Goal 1" --amount 250
i-rs-goal deposit "Test Goal 1" --amount 350
```

### Add milestone
```bash
i-rs-goal milestone -g "Test Goal 1" -n "First 500" -a 500
```

### List milestones
```bash
i-rs-goal milestone -g "Test Goal 1" --list
```

### View statistics
```bash
i-rs-goal stats
```

### Update goal
```bash
i-rs-goal update "Test Goal 1" --target 1500
```

### Delete test goals
```bash
i-rs-goal delete "Test Goal 1"
i-rs-goal delete "Test Goal 2"
i-rs-goal delete "Test Goal 3"
```

## Expected Results

### After deposits to Test Goal 1
- Current amount: 600.00
- Progress: 60.0% (of 1000)
- Remaining: 400.00

### After milestone add
- Milestone "First 500" should be marked as reached

### Statistics
- Should show 3 goals
- Total target: 16500
- Total current: 600 (after deposits)
