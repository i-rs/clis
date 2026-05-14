# i-rs-step Examples

## Basic Usage

### Recording Steps

```bash
# Simple recording
i-rs-step add 10000
i-rs-step add 8500
i-rs-step add 12000

# With distance
i-rs-step add 10000 --distance 8.0
i-rs-step add 8000 --distance 6.4
```

### With Tags

```bash
# Tagged entries
i-rs-step add 10000 --tag workday
i-rs-step add 15000 --tag weekend --tag hiking
i-rs-step add 5000 --tag rest-day
```

## Viewing Records

```bash
# List all records
i-rs-step list

# Get specific date
i-rs-step get 2024-01-15

# Delete old record
i-rs-step delete 2024-01-01
```

## Fitness Tracking

```bash
# Daily goals
i-rs-step add 10000 --tag goal-10k --tag weekday
i-rs-step add 15000 --tag goal-10k --tag weekend

# Running days
i-rs-step add 8000 --distance 6.0 --tag running --tag exercise

# Walking days
i-rs-step add 12000 --distance 9.6 --tag walking --tag exercise

# Rest days
i-rs-step add 3000 --tag rest-day --tag recovery
```

## Integration Examples

### Weekly Summary Script

```bash
#!/bin/bash
# weekly-steps.sh - Check weekly step total

echo "Weekly Step Report"
echo "================="

total=0
for i in {0..6}; do
    date=$(date -v-$i +%Y-%m-%d)
    steps=$(i-rs-step get $date 2>/dev/null | grep steps | awk '{print $2}')
    if [ -n "$steps" ]; then
        total=$((total + steps))
        echo "$date: $steps steps"
    fi
done

echo "Total: $total steps"
```