# i-rs-walkdog Examples

## Basic Usage

### Recording Walks

```bash
# Quick walks
i-rs-walkdog add "Buddy" 15
i-rs-walkdog add "Buddy" 30

# Regular walks
i-rs-walkdog add "Buddy" 45
i-rs-walkdog add "Max" 30
```

### With Tags

```bash
# Tagged by time
i-rs-walkdog add "Buddy" 30 --tag morning
i-rs-walkdog add "Buddy" 45 --tag evening
i-rs-walkdog add "Buddy" 15 --tag quick

# Park visits
i-rs-walkdog add "Buddy" 60 --tag park
```

## Viewing Records

```bash
# List all records
i-rs-walkdog list

# Filter by tag
i-rs-walkdog list --tag morning
i-rs-walkdog list --tag park
```

## Managing Records

```bash
# Get record details
i-rs-walkdog get abc12345

# Delete a record
i-rs-walkdog delete abc12345
```

## Daily Routine

```bash
# Morning walk
i-rs-walkdog add "Buddy" 30 --tag morning --tag before-breakfast

# Evening walk
i-rs-walkdog add "Buddy" 45 --tag evening --tag after-work

# Weekend long walk
i-rs-walkdog add "Buddy" 90 --tag weekend --tag park
```

## Integration Examples

### Daily Walk Script

```bash
#!/bin/bash
# walk-dog.sh - Log a dog walk

echo "Recording a dog walk..."
read -p "Dog name: " dog
read -p "Duration (minutes): " duration

i-rs-walkdog add "$dog" $duration

echo "Logged ${duration} minute walk for $dog!"
```