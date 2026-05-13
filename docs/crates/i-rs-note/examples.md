# i-rs-note Examples

## Basic Usage

### Creating Notes

```bash
# Simple shopping list
i-rs-note add shopping --title "Shopping List" --content "Milk" --content "Bread" --content "Eggs"

# Todo list
i-rs-note add todo --title "Today's Tasks" --tag work --tag daily --content "Review PRs" --content "Team meeting" --content "Write documentation"

# Project notes
i-rs-note add project-alpha --title "Project Alpha" --tag project --tag alpha --content "Kickoff: Jan 15" --content "Deadline: Mar 30"

# Meeting notes
i-rs-notes add meeting-2024-01-15 --title "Weekly Standup" --tag meeting --tag standup --content "Updates from team" --content "Blockers" --content "Next steps"
```

### Managing Notes

```bash
# List all notes
i-rs-note list

# List by tag
i-rs-note list --tag work
i-rs-note list --tag project
i-rs-note list --tag meeting

# Get note content
i-rs-note get shopping
i-rs-note get todo
```

### Updating Notes

```bash
# Add more content
i-rs-note add todo --content "New task"

# Update title
i-rs-note update shopping --title "Weekly Shopping"

# Replace content
i-rs-note update todo --content "Updated task 1" --content "Updated task 2"

# Update tags
i-rs-note update project-alpha --tag alpha --tag important
```

### Deleting Notes

```bash
i-rs-note delete old-note
i-rs-note delete meeting-notes
```

## Advanced Usage

### Todo List Workflow

```bash
# Create todo list
i-rs-note add today-todo --title "Today's Work" --tag todo --tag daily

# View current tasks
i-rs-note get today-todo

# Add more tasks
i-rs-note update today-todo --content "Code review" --content "Write tests" --content "Update documentation"

# Clear and start fresh (by updating with new content)
i-rs-note update today-todo --content "New tasks for today"
```

### Project Documentation

```bash
# Create project note
i-rs-note add project-notes --title "My Project" --tag project --content "Phase 1: Planning" --content "Phase 2: Development" --content "Phase 3: Testing"

# Add milestones
i-rs-note update project-notes --content "Phase 1: Planning" --content "Phase 2: Development" --content "Phase 3: Testing" --content "Milestone 1: Complete spec" --content "Milestone 2: MVP ready"
```

### Meeting Notes

```bash
# Quick meeting note
i-rs-note add standup-2024-01-15 --title "Jan 15 Standup" --tag meeting --content "John: Completed login" --content "Jane: Working on dashboard" --content "Bob: Blocked by API"

# List meeting notes
i-rs-note list --tag meeting
```

### Reference Notes

```bash
# API reference
i-rs-note add api-cheatsheet --title "API Cheatsheet" --tag reference --tag api --content "GET /users - List users" --content "POST /users - Create user" --content "GET /users/:id - Get user"

# Command reference
i-rs-note add docker-commands --title "Docker Commands" --tag reference --tag docker --content "docker ps - List containers" --content "docker run - Run container" --content "docker build - Build image"
```