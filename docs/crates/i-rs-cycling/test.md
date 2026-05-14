# i-rs-cycling Test Records

## Setup

```bash
# Add test records
i-rs-cycling add 2025-06-14 25.5 60 --elevation 300 --route "Lakeside Loop" --tag morning --tag easy
i-rs-cycling add 2025-06-15 30.2 75 --elevation 450 --route "Mountain Trail" --tag mountain --tag training
i-rs-cycling add 2025-06-16 15.0 30 --route "Park Trail" --tag morning
i-rs-cycling add 2025-06-17 40.0 90 --elevation 200 --route "River Road" --tag weekend --tag long-ride
```

## Test Commands

### List Records
```bash
i-rs-cycling list
```

Expected output: Table with 4 records

### Filter by Tag
```bash
i-rs-cycling list --tag mountain
```

Expected output: Only the mountain trail record

### Get Record
```bash
i-rs-cycling get 2025-06-14
```

Expected output: Details of the first record

### View Statistics
```bash
i-rs-cycling stats
```

Expected output:
```
Statistics:
  Records:       4 records
  Total Distance: 110.7 km
  Total Duration: 255 min (4.3 h)
  Total Elevation: 950 m
  Avg Speed:     26.0 km/h
```

### JSON Output
```bash
i-rs-cycling list --json
```

Expected output: JSON array with all records

### Update Record
```bash
# Get the UUID first
i-rs-cycling get 2025-06-14

# Update distance
i-rs-cycling update <uuid> --distance 26.0

# Add a tag
i-rs-cycling update 2025-06-14 --add-tag favorite
```

### Delete Record
```bash
# Delete by date
i-rs-cycling delete 2025-06-16

# Verify deletion
i-rs-cycling list
```

Expected output: 3 records remaining
