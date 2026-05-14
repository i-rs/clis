# i-rs-goal Examples

## Basic Usage

### Create a savings goal

```bash
i-rs-goal add "Emergency Fund" --target 10000 --deadline 2025-12-31
```

### Create a goal with tags

```bash
i-rs-goal add "Vacation Fund" --target 5000 --deadline 2025-06-01 --tags travel,leisure
```

### Create a goal with milestones

```bash
i-rs-goal add "House Fund" --target 100000 --deadline 2026-12-31 \
  --milestones "First 10k:10000,First 25k:25000,First 50k:50000"
```

### Create a goal with remarks

```bash
i-rs-goal add "Car Fund" --target 30000 --deadline 2025-12-31 \
  --tags car,transport --remark "Saving for down payment","Monthly contribution: 500"
```

## Tracking Progress

### Deposit to a goal

```bash
i-rs-goal deposit "Emergency Fund" --amount 500
```

### Deposit multiple times

```bash
i-rs-goal deposit "Emergency Fund" --amount 1000
i-rs-goal deposit "Emergency Fund" --amount 750
```

### View goal details

```bash
i-rs-goal get "Emergency Fund"
```

Output:
```
Goal: Emergency Fund

Target Amount: 10000.00
Current Amount: 2250.00
Progress: 22.5%
Remaining: 7750.00
Deadline: 2025-12-31
Days Left: 230

Milestones:
┌─────────────┬──────────┬───────────┬─────────────────┐
│ Name        │ Amount   │ Status    │ Reached At      │
├─────────────┼──────────┼───────────┼─────────────────┤
│ First 1000  │ 1000.00  │ ✓ Reached │ 2025-01-15 10:30│
│ First 2500  │ 2500.00  │ ○ Pending │ -               │
└─────────────┴──────────┴───────────┴─────────────────┘
```

## Managing Milestones

### Add a milestone to existing goal

```bash
i-rs-goal milestone -g "Emergency Fund" -n "Half Way" -a 5000
```

### List milestones

```bash
i-rs-goal milestone -g "Emergency Fund" --list
```

### Remove a milestone

```bash
i-rs-goal milestone -g "Emergency Fund" -r <milestone-id>
```

## Organizing with Tags

### List goals by tag

```bash
i-rs-goal list --tag emergency
i-rs-goal list --tag travel
```

### View statistics by tag

```bash
i-rs-goal stats --tag emergency
```

## Updating Goals

### Update target amount

```bash
i-rs-goal update "Emergency Fund" --target 15000
```

### Update deadline

```bash
i-rs-goal update "Emergency Fund" --deadline 2026-06-30
```

### Update tags

```bash
i-rs-goal update "Emergency Fund" --tags emergency,high-priority,finance
```

## Viewing Statistics

### View overall statistics

```bash
i-rs-goal stats
```

Output:
```
Savings Statistics

Overall Progress: 35.2%
████████████████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░

Total Saved: 15750.00 / 44750.00
Remaining: 29000.00

Summary: Goals
  ✓ 2 completed
  ○ 5 on track
  ✗ 1 overdue

Statistics: By Tag
  emergency: 45.2% (4520.00 / 10000.00)
  travel: 100.0% (5000.00 / 5000.00)
  car: 20.8% (6250.00 / 30000.00)
```

## Deleting Goals

### Delete a goal

```bash
i-rs-goal delete "Old Goal"
```

## JSON Output

All commands support `--json` flag for scripting:

```bash
i-rs-goal list --json
i-rs-goal get "Emergency Fund" --json
i-rs-goal stats --json
i-rs-goal milestone -g "Emergency Fund" --list --json
```

### Example JSON output for list

```json
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "name": "Emergency Fund",
      "target_amount": 10000,
      "current_amount": 2500,
      "progress_percentage": 25.0,
      "deadline": "2025-12-31T00:00:00Z",
      "tags": ["emergency", "finance"],
      "milestones": [...]
    }
  ],
  "meta": {
    "count": 1,
    "filter": null
  }
}
```

## Practical Scenarios

### Scenario 1: Emergency Fund

```bash
# Create emergency fund
i-rs-goal add "Emergency Fund" \
  --target 10000 \
  --deadline 2025-12-31 \
  --tags emergency,finance \
  --milestones "First 1k:1000,First 5k:5000"

# Monthly deposits
i-rs-goal deposit "Emergency Fund" --amount 500

# Check progress
i-rs-goal get "Emergency Fund"
```

### Scenario 2: Vacation Planning

```bash
# Create vacation fund
i-rs-goal add "Japan Trip 2025" \
  --target 20000 \
  --deadline 2025-04-01 \
  --tags travel,japan

# Add milestone
i-rs-goal milestone -g "Japan Trip 2025" -n "Flight Booked" -a 8000

# When flight is booked
i-rs-goal deposit "Japan Trip 2025" --amount 8000
```

### Scenario 3: Multiple Goals Tracking

```bash
# Create multiple goals
i-rs-goal add "Emergency Fund" --target 10000 --deadline 2025-12-31 --tags emergency
i-rs-goal add "Vacation" --target 5000 --deadline 2025-06-01 --tags travel
i-rs-goal add "New Car" --target 30000 --deadline 2026-12-31 --tags car

# View all goals
i-rs-goal list

# View statistics
i-rs-goal stats
```
