# i-rs-height Examples

## Basic Usage

### Add Height Records

```bash
# Basic height record
i-rs-height add 2025-06-14 175.5

# With weight
i-rs-height add 2025-06-14 175.5 --weight 68.5

# With tags
i-rs-height add 2025-06-14 175.5 --tag "morning" --tag "before-workout"

# With remarks
i-rs-height add 2025-06-14 175.5 --remark "Measured in the morning"
```

### View Records

```bash
# List all records
i-rs-height list

# List last 30 days
i-rs-height list --days 30

# List with ASCII chart
i-rs-height list --chart

# List with statistics
i-rs-height list --stats

# Combined options
i-rs-height list --days 7 --chart --stats
```

### Get Specific Record

```bash
# Get record by date
i-rs-height get 2025-06-14

# Get in JSON format
i-rs-height get 2025-06-14 --json
```

### Delete Records

```bash
# Delete a record
i-rs-height delete 2025-06-15
```

## Target Height

```bash
# Set target height
i-rs-height set 180.0

# Show current target
i-rs-height target
```

## Statistics Display

When using `--stats` flag, you'll see:
- **Min**: Minimum recorded height
- **Max**: Maximum recorded height
- **Average**: Average height
- **Change**: Total height change from first to last record
- **Target**: Goal height (if set)
- **Gap**: Difference from target

## Chart Visualization

The ASCII chart shows:
- Height values plotted over time
- Start → End indicators
- Change summary with direction arrows (↑ ↓ →)

## JSON Output

```bash
# List in JSON format
i-rs-height list --json

# Get specific record in JSON
i-rs-height get 2025-06-14 --json
```

## Date Formats

Supported date formats:
- `YYYY-MM-DD` (recommended)
- `YYYY/MM/DD`
- `DD-MM-YYYY`
- `DD/MM/YYYY`

## Use Cases

### Growth Tracking

```bash
# Track child's growth
i-rs-height add 2025-01-01 140.0
i-rs-height add 2025-02-01 141.0
i-rs-height add 2025-03-01 142.0
i-rs-height list --chart
```

### Weight Correlation

```bash
# Track both height and weight
i-rs-height add 2025-06-14 175.5 --weight 68.5
i-rs-height add 2025-06-21 175.8 --weight 68.0
i-rs-height add 2025-06-28 176.0 --weight 67.5
```

### Morning Measurements

```bash
# Consistent morning measurements
i-rs-height add 2025-06-14 175.5 --tag "morning" --remark "Before breakfast"
```
