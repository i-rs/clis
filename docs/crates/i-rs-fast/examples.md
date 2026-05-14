# i-rs-fast Examples

## Basic Usage

### Starting Fasts

```bash
# Common fasting protocols
i-rs-fast add 16
i-rs-fast add 18
i-rs-fast add 20

# OMAD (One Meal A Day)
i-rs-fast add 23 --tag omad
```

### With Tags

```bash
# Tagged entries
i-rs-fast add 16 --tag 16:8
i-rs-fast add 18 --tag 18:6
i-rs-fast add 23 --tag omad
i-rs-fast add 24 --tag extended
```

## Viewing Records

```bash
# List all records
i-rs-fast list

# Filter by tag
i-rs-fast list --tag omad
i-rs-fast list --tag extended
```

## Managing Records

```bash
# Get record details
i-rs-fast get abc12345

# Delete a record
i-rs-fast delete abc12345
```

## Fasting Journal

```bash
# Morning start (16:8)
i-rs-fast add 16 --tag 16:8 --remark "First fast"

# Extended fast
i-rs-fast add 24 --tag extended --remark "Weekend detox"

# OMAD
i-rs-fast add 23 --tag omad --remark "Daily practice"
```

## Integration Examples

### Daily Fasting Script

```bash
#!/bin/bash
# start-fast.sh - Start a fasting session

echo "Starting a fast..."
echo "Common protocols:"
echo "1: 16:8 (16 hours)"
echo "2: 18:6 (18 hours)"
echo "3: 20:4 (20 hours)"
echo "4: OMAD (23 hours)"
echo "5: Extended (24+ hours)"

read -p "Choose protocol (1-5): " choice

case $choice in
    1) i-rs-fast add 16 --tag 16:8 ;;
    2) i-rs-fast add 18 --tag 18:6 ;;
    3) i-rs-fast add 20 --tag 20:4 ;;
    4) i-rs-fast add 23 --tag omad ;;
    5) read -p "Enter hours: " hours
       i-rs-fast add $hours --tag extended ;;
    *) echo "Invalid choice"; exit 1 ;;
esac

echo "Fasting session started!"
```