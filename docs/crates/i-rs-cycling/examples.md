# i-rs-cycling Examples

## Basic Usage

### Add a Simple Record
```bash
i-rs-cycling add 2025-06-14 25.5 60
```

### Add a Record with Elevation
```bash
i-rs-cycling add 2025-06-15 30.2 75 --elevation 450
```

### Add a Record with Route and Tags
```bash
i-rs-cycling add 2025-06-16 15.0 30 \
  --route "Morning ride around the lake" \
  --tag morning \
  --tag easy
```

### Add a Record with Multiple Tags
```bash
i-rs-cycling add 2025-06-17 40.0 90 \
  --elevation 200 \
  --tag mountain \
  --tag training \
  --tag weekend
```

### Add a Record with Remarks
```bash
i-rs-cycling add 2025-06-18 20.0 45 \
  --remark "Beautiful weather" \
  --remark "Met friends on the trail"
```

## Viewing Records

### List All Records
```bash
i-rs-cycling list
```

### List Records Filtered by Tag
```bash
i-rs-cycling list --tag mountain
i-rs-cycling list --tag weekend
```

### View Record Details
```bash
i-rs-cycling get 550e8400-e29b-41d4-a716-446655440000
i-rs-cycling get 2025-06-14
```

### Get Record as JSON
```bash
i-rs-cycling get 550e8400-e29b-41d4-a716-446655440000 --json
i-rs-cycling list --json
```

## Updating Records

### Update Distance
```bash
i-rs-cycling update <uuid> --distance 26.0
```

### Update Duration
```bash
i-rs-cycling update 2025-06-14 --duration 65
```

### Update Elevation
```bash
i-rs-cycling update <uuid> --elevation 350
```

### Add a Tag
```bash
i-rs-cycling update <uuid> --add-tag favorite
```

### Remove a Tag
```bash
i-rs-cycling update <uuid> --remove-tag morning
```

### Update Route
```bash
i-rs-cycling update <uuid> --route "New scenic route"
```

### Add a Remark
```bash
i-rs-cycling update <uuid> --add-remark "Perfect conditions today"
```

## Deleting Records

### Delete by UUID
```bash
i-rs-cycling delete 550e8400-e29b-41d4-a716-446655440000
```

### Delete by Date
```bash
i-rs-cycling delete 2025-06-14
```

## Statistics

### View All Statistics
```bash
i-rs-cycling stats
```

Output:
```
Statistics:
  Records:       10 records
  Total Distance: 250.5 km
  Total Duration: 300 min (5.0 h)
  Total Elevation: 2500 m
  Avg Speed:     25.5 km/h
  Avg Distance:  25.05 km
  Avg Duration:  30 min
```

## Workflow Examples

### Morning Ride Routine
```bash
# Add morning ride
i-rs-cycling add 2025-06-20 15.0 35 \
  --route "Morning commute" \
  --tag morning \
  --tag commute

# View this week's stats
i-rs-cycling stats
```

### Weekend Long Ride
```bash
# Record a long weekend ride
i-rs-cycling add 2025-06-21 60.0 150 \
  --elevation 800 \
  --route "Mountain pass loop" \
  --tag mountain \
  --tag long-ride \
  --tag weekend \
  --remark "Challenging but rewarding"

# List all mountain rides
i-rs-cycling list --tag mountain
```

### Training Progress
```bash
# Add training ride
i-rs-cycling add 2025-06-22 30.0 60 \
  --elevation 150 \
  --tag training \
  --tag interval

# Update weekly stats
i-rs-cycling stats

# List training rides
i-rs-cycling list --tag training
```
