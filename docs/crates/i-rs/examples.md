# Examples

## Basic Tool Usage

```bash
# List todos
i-rs todo list

# Add a todo with priority
i-rs todo add "Buy groceries" --priority high

# Record mood for today
i-rs mood add today happy

# Log weight
i-rs weight add 75.5
```

## Getting Help

```bash
# List available tools
i-rs

# Show help for a specific tool
i-rs todo --help
i-rs mood --help
i-rs weight --help

# Show version
i-rs --version
```

## Finance Tools

```bash
# Record an expense
i-rs ledger add "Lunch" --amount 25 --tag food

# Check budget status
i-rs budget stats

# View investment portfolio
i-rs invest list
```

## Health Tracking

```bash
# Log water intake
i-rs water add 500

# View sleep statistics
i-rs sleep stats

# Log exercise
i-rs exercise add "Morning Run" --duration 30 --calories 250
```

## Productivity

```bash
# Capture a quick note
i-rs note add "Meeting notes" --content "Discussed Q2 roadmap"

# Save a bookmark
i-rs bookmark add "GitHub" --url https://github.com

# Track a habit
i-rs habit checkin "Daily Walk"
```

## Data Management

Each tool supports data export, import, and clear operations:

```bash
# Export all todos
i-rs todo data export

# Import data from file
i-rs todo data import backup.json

# Clear all data
i-rs todo data clear
```
