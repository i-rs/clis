# i-rs-height Test Records

Use these test commands to verify the installation and functionality.

## Test Sequence

### 1. Add Test Records

```bash
# Add first record
i-rs-height add 2025-06-10 175.0

# Add second record
i-rs-height add 2025-06-11 175.2

# Add third record with weight
i-rs-height add 2025-06-12 175.5 --weight 68.0

# Add fourth record with tags
i-rs-height add 2025-06-13 175.8 --tag "morning" --remark "After breakfast"
```

### 2. List Records

```bash
# List all
i-rs-height list

# List with chart
i-rs-height list --chart

# List with stats
i-rs-height list --stats
```

### 3. Get Specific Record

```bash
i-rs-height get 2025-06-12
```

### 4. Set Target

```bash
i-rs-height set 180.0
i-rs-height target
```

### 5. Delete Test Records

```bash
i-rs-height delete 2025-06-13
i-rs-height delete 2025-06-12
i-rs-height delete 2025-06-11
i-rs-height delete 2025-06-10
```

## Expected Output

### List Output
```
┌────────────┬──────────┬──────────┬─────────┬──────────────────────┐
│ DATE       │ HEIGHT   │ WEIGHT   │ TAGS    │ REMARK               │
├────────────┼──────────┼──────────┼─────────┼──────────────────────┤
│ 2025-06-10 │ 175.0    │ -        │ -       │ -                    │
│ 2025-06-11 │ 175.2    │ -        │ -       │ -                    │
│ 2025-06-12 │ 175.5    │ 68.0     │ -       │ -                    │
│ 2025-06-13 │ 175.8    │ -        │ morning │ After breakfast      │
└────────────┴──────────┴──────────┴─────────┴──────────────────────┘

Total: 4 records
```

### Stats Output
```
Statistics:
Min:        175.0 cm
Max:        175.8 cm
Average:    175.4 cm
Change:     +0.8 cm
Target:     180.0 cm
Gap:        +4.2 cm to target
```
