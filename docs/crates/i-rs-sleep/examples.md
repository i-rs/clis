# i-rs-sleep Examples

## Basic Usage

### Recording Sleep

```bash
# Record a typical workday sleep
i-rs-sleep add 22:30 06:45 4 --tag workday

# Record a great weekend sleep
i-rs-sleep add 23:00 08:30 5 --tag weekend --remark "Perfect sleep!"

# Record a poor night's sleep
i-rs-sleep add 01:00 06:00 2 --tag workday --remark "Insomnia"
```

### Viewing Records

```bash
# List all sleep records
i-rs-sleep list

# Filter by tag
i-rs-sleep list --tag weekend

# Get detailed information
i-rs-sleep get abc12345

# Get JSON output
i-rs-sleep list --json
```

### Statistics

```bash
# View sleep statistics
i-rs-sleep stats
```

### Updating Records

```bash
# Update quality rating
i-rs-sleep update abc12345 --quality 4

# Update bedtime
i-rs-sleep update abc12345 --bedtime 22:45

# Update multiple fields
i-rs-sleep update abc12345 --bedtime 22:00 --wake-time 07:00 --quality 5
```

### Deleting Records

```bash
# Delete a record
i-rs-sleep delete abc12345
```

## Output Examples

### List Output

```
╭──────────────────────────────────────┬───────────────┬──────┬───────┬─────────┬──────────╮
│ ID                                 │ BEDTIME       │ WAKE │ HOURS │ QUALITY │ TAGS     │
├──────────────────────────────────────┼───────────────┼──────┼───────┼─────────┼──────────┤
│ a1b2c3d4-e5f6-7890-1234-56789abcdef│ 2024-01-15    │ 06:45│ 8.2h  │ 😊 Good │ workday  │
│                                      22:30         │      │       │         │          │
│ 09876543-210f-edcb-a987-6543210fedc│ 2024-01-14    │ 08:30│ 9.5h  │ 😁 Exc  │ weekend  │
│                                      23:00         │      │       │ ellent  │          │
╰──────────────────────────────────────┴───────────────┴──────┴───────┴─────────┴──────────╯

Total: 2 records
```

### Statistics Output

```
Sleep Statistics
Total records: 30
Average duration: 7.8h
Average quality: 3.9/5
Min duration: 5.5h
Max duration: 9.5h
```