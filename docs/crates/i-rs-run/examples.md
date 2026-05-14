# Examples

## Basic Run Recording

### Simple Run
```bash
i-rs-run add 2025-06-14 5.0 30
```

### Run with Heart Rate
```bash
i-rs-run add 2025-06-15 10.0 60 -r 155
```

### Run with Weather
```bash
i-rs-run add 2025-06-16 8.0 45 -w "Cloudy, 18°C"
```

### Run with Tags
```bash
i-rs-run add 2025-06-17 5.0 25 -t interval -t speedwork
```

### Full Example
```bash
i-rs-run add 2025-06-18 15.0 90 -r 145 -w sunny -t marathon -t long-run --remark "Felt great!"
```

## Viewing Records

### List All Runs
```bash
i-rs-run list
```

### Get Run Details
```bash
i-rs-run get abc123-def456
```

### Get Details in JSON
```bash
i-rs-run get abc123-def456 --json
```

## Statistics

### View Cumulative Stats
```bash
i-rs-run stats
```

Shows:
- Total distance
- Total duration
- Average pace
- Number of records

## Managing Plans

### Create Training Plan
```bash
i-rs-run plan-add "Marathon Prep" 42.0 5:30 --schedule 0 2 4 6 -t marathon
```

### Weekly Training Plan
```bash
i-rs-run plan-add "5K Training" 5.0 6:00 --schedule 1 3 5
```

### List All Plans
```bash
i-rs-run plan-list
```

### View Plan Details
```bash
i-rs-run plan-get xyz789
```

### Delete a Plan
```bash
i-rs-run plan-delete xyz789
```

## Deleting Records

### Delete a Run
```bash
i-rs-run delete abc123-def456
```

## JSON Output

All commands support `--json` for machine-readable output:

```bash
i-rs-run list --json
i-rs-run stats --json
i-rs-run plan-list --json
```
