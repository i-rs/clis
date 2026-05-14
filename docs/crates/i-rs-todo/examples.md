# i-rs-todo Examples

## Basic Usage

### Creating Todos

```bash
# Simple todo
i-rs-todo add todo-1 --title "Buy groceries"

# With priority
i-rs-todo add todo-2 --title "Submit report" --priority high
i-rs-todo add todo-3 --title "Read book" --priority low

# With tags
i-rs-todo add todo-4 --title "Call mom" --tag personal --tag important

# With content
i-rs-todo add todo-5 --title "Project plan" --tag work --content "Define scope" --content "Set milestones"
```

### Managing Todos

```bash
# List all
i-rs-todo list

# Mark as done
i-rs-todo done todo-1

# Mark as pending (toggle back)
i-rs-todo done todo-1

# Get details
i-rs-todo get todo-1

# Update
i-rs-todo update todo-1 --title "Updated title" --priority high

# Delete
i-rs-todo delete todo-1
```

## Filtering

### By Status

```bash
# List pending only
i-rs-todo list --pending

# List done only
i-rs-todo list --done

# List all (default)
i-rs-todo list
```

### By Tag

```bash
# Filter by tag
i-rs-todo list --tag work

# Combined with status
i-rs-todo list --tag work --pending
```

## Priority Examples

### High Priority

```bash
i-rs-todo add urgent-1 --title "Fix production bug" --priority high --tag critical
i-rs-todo add urgent-2 --title "Client meeting" --priority high --tag work
```

### Medium Priority

```bash
i-rs-todo add mid-1 --title "Reply emails" --priority medium
i-rs-todo add mid-2 --title "Update documentation" --priority medium --tag work
```

### Low Priority

```bash
i-rs-todo add low-1 --title "Organize files" --priority low
i-rs-todo add low-2 --title "Clean desk" --priority low --tag personal
```

## Project Management

### Work Project

```bash
# Create project todos
i-rs-todo add proj-design --title "Design mockups" --priority high --tag project-x --content "Create wireframes"
i-rs-todo add proj-dev --title "Implement feature" --priority high --tag project-x --content "Use React"
i-rs-todo add proj-test --title "Write tests" --priority medium --tag project-x
i-rs-todo add proj-docs --title "Documentation" --priority low --tag project-x

# Track progress
i-rs-todo list --tag project-x
i-rs-todo done proj-design
i-rs-todo done proj-dev
```

### Daily Routine

```bash
# Morning routine
i-rs-todo add routine-1 --title "Check emails" --priority medium --tag daily --tag work
i-rs-todo add routine-2 --title "Team standup" --priority high --tag daily --tag work --content "9:30 AM"
i-rs-todo add routine-3 --title "Review PRs" --priority medium --tag daily --tag work

# Mark as done throughout the day
i-rs-todo done routine-1
i-rs-todo done routine-2
i-rs-todo done routine-3
```

## Personal Tasks

### Shopping List

```bash
i-rs-todo add shop-1 --title "Groceries" --tag shopping --content "Milk" --content "Bread" --content "Eggs"
i-rs-todo add shop-2 --title "Hardware store" --tag shopping --content "Screws" --content "Paint"
i-rs-todo add shop-3 --title "Pharmacy" --tag shopping --priority medium --content "Vitamins"
```

### Home Maintenance

```bash
i-rs-todo add home-1 --title "Fix leaky faucet" --priority medium --tag home --tag repair
i-rs-todo add home-2 --title "Paint bedroom" --priority low --tag home --content "Choose color first"
i-rs-todo add home-3 --title "Clean gutters" --priority high --tag home --tag urgent
```

## Advanced Usage

### Task Dependencies

```bash
# Define dependencies
i-rs-todo add step-1 --title "Design DB schema" --priority high --tag project
i-rs-todo add step-2 --title "Setup database" --priority high --tag project --content "Depends on schema"
i-rs-todo add step-3 --title "Implement API" --priority high --tag project --content "Depends on DB"

# Mark step-1 done, then step-2, then step-3
i-rs-todo done step-1
i-rs-todo done step-2
i-rs-todo done step-3
```

### Time Blocking

```bash
i-rs-todo add block-1 --title "Deep work session" --priority high --tag focus --content "2 hours uninterrupted"
i-rs-todo add block-2 --title "Code review" --priority medium --tag focus --content "1 hour"
i-rs-todo add block-3 --title "Planning" --priority low --tag focus --content "30 minutes"
```

### Recurring Tasks

```bash
# Weekly review
i-rs-todo add weekly-1 --title "Weekly review" --priority medium --tag recurring --content "Review todos for next week"

# Daily standup preparation
i-rs-todo add standup-1 --title "Prepare standup" --priority medium --tag daily --content "Update progress notes"
```

## Integration Examples

### Daily Todo Script

```bash
#!/bin/bash
# add-todo.sh - Quick add todo

NAME=$1
shift

i-rs-todo add "$NAME" "$@"

echo "Todo '$NAME' added!"
```

### Weekly Review Script

```bash
#!/bin/bash
# weekly-review.sh - Weekly todo review

echo "=== Pending Todos ==="
i-rs-todo list --pending

echo ""
echo "=== Done This Week ==="
i-rs-todo list --done
```

### Clear Completed

```bash
#!/bin/bash
# clear-done.sh - Remove all done todos

echo "Current done todos:"
i-rs-todo list --done

read -p "Delete all done todos? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Note: This requires iterating - simplified here
    echo "Please delete done todos individually or by name pattern"
fi
```

## Data Export

### View Raw Data

```bash
cat ~/.config/i-rs/todos.json
```

### Backup Script

```bash
#!/bin/bash
# backup-todos.sh

BACKUP_DIR=~/backups/todos
mkdir -p $BACKUP_DIR

DATE=$(date +%Y-%m-%d)
cp ~/.config/i-rs/todos.json "$BACKUP_DIR/todos-$DATE.json"

echo "Backup saved to $BACKUP_DIR/todos-$DATE.json"
```
