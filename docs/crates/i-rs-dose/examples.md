# i-rs-dose Examples

## Basic Usage

### Recording Medicine Intake

```bash
# Record vitamins
i-rs-dose add "Vitamin D" --dosage 1000 --unit IU
i-rs-dose add "Vitamin C" --dosage 500 --unit mg

# Record medications
i-rs-dose add "Ibuprofen" --dosage 400 --unit mg
i-rs-dose add "Aspirin" --dosage 100 --unit mg
```

### With Tags and Remarks

```bash
# With tags
i-rs-dose add "Vitamin B12" --dosage 1000 --unit mcg --tag morning --tag supplement

# With remarks
i-rs-dose add "Iron Supplement" --dosage 65 --unit mg --remark "Take with food"
```

## Viewing Records

```bash
# List all records
i-rs-dose list

# Filter by tag
i-rs-dose list --tag morning

# Get details
i-rs-dose get abc12345
```

## Managing Records

```bash
# Delete old record
i-rs-dose delete abc12345
```

## Daily Supplement Routine

```bash
# Morning vitamins
i-rs-dose add "Vitamin D" --dosage 2000 --unit IU --tag morning --tag daily
i-rs-dose add "Vitamin B Complex" --dosage 1 --unit tablet --tag morning --tag daily

# With food
i-rs-dose add "Fish Oil" --dosage 1000 --unit mg --tag with-food --tag daily

# Evening
i-rs-dose add "Magnesium" --dosage 400 --unit mg --tag evening --tag sleep
```