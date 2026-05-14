# i-rs-project Test Records

Test records for verifying i-rs-project functionality.

## Test 1: Basic Project Operations

### Create Projects

```bash
# Create first project
i-rs-project add test-project-1 --description "First test project"

# Create second project with priority
i-rs-project add test-project-2 --priority high --tag test

# Create third project
i-rs-project add test-project-3 --status onhold
```

### List Projects

```bash
# List all projects
i-rs-project list

# Verify output contains all 3 projects
# Expected: test-project-1, test-project-2, test-project-3
```

### Get Project Details

```bash
# Get first project details
i-rs-project get test-project-1

# Verify: shows name, description, status, priority, milestones, tasks
```

### Update Project

```bash
# Update project status
i-rs-project update test-project-1 --status completed

# Verify: status changed to completed
i-rs-project get test-project-1
```

## Test 2: Milestone Management

### Add Milestones

```bash
# Add milestone to project 2
i-rs-project milestone add test-project-2 "v1.0" --due-date 2026-06-01

# Add another milestone
i-rs-project milestone add test-project-2 "Beta" --description "Beta release" --due-date 2026-05-20

# Verify milestones
i-rs-project get test-project-2
# Expected: 2 milestones listed
```

### Complete Milestone

```bash
# Complete first milestone
i-rs-project milestone complete test-project-2 "v1.0"

# Verify: milestone marked as completed
i-rs-project get test-project-2
```

## Test 3: Task Management

### Add Tasks

```bash
# Add task to project 2
i-rs-project task add test-project-2 "Write tests" --tag testing

# Add another task
i-rs-project task add test-project-2 "Review PR" --description "Review pull requests"

# Add more tasks
i-rs-project task add test-project-2 "Update docs"
i-rs-project task add test-project-2 "Deploy" --tag devops

# Verify tasks
i-rs-project get test-project-2
# Expected: 4 tasks listed
```

### Complete Tasks

```bash
# Complete a task
i-rs-project task complete test-project-2 "Write tests"

# Complete another task
i-rs-project task complete test-project-2 "Update docs"

# Verify: tasks marked as completed
i-rs-project get test-project-2
# Expected: 2 completed, 2 remaining
```

## Test 4: Filtering

### Filter by Tag

```bash
# List projects with tag "test"
i-rs-project list --tag test

# Verify: only test-project-2 shown
```

### Filter by Status

```bash
# List completed projects
i-rs-project list --status completed

# Verify: test-project-1 shown
```

## Test 5: Statistics

```bash
# View statistics
i-rs-project stats

# Verify: shows project counts, milestone counts, task completion rates
```

## Test 6: JSON Output

```bash
# List in JSON
i-rs-project list --json

# Get in JSON
i-rs-project get test-project-2 --json

# Stats in JSON
i-rs-project stats --json

# Verify: valid JSON output for all commands
```

## Test 7: Error Handling

### Non-existent Project

```bash
# Try to get non-existent project
i-rs-project get non-existent

# Expected: Error message "Project 'non-existent' not found"
```

### Duplicate Milestone

```bash
# Try to add duplicate milestone
i-rs-project milestone add test-project-2 "v1.0"

# Expected: Error message "Milestone 'v1.0' already exists"
```

### Milestone in Non-existent Project

```bash
# Try to add milestone to non-existent project
i-rs-project milestone add non-existent "test" --due-date 2026-06-01

# Expected: Error message "Project 'non-existent' not found"
```

## Test 8: Example and Skill Commands

```bash
# Show examples
i-rs-project example

# Show skill summary
i-rs-project skill summary

# Show skill content
i-rs-project skill content

# Verify: help content displayed correctly
```

## Cleanup

```bash
# Delete test projects
i-rs-project delete test-project-1
i-rs-project delete test-project-2
i-rs-project delete test-project-3

# Verify: projects deleted
i-rs-project list

# Expected: no projects listed
```

## Test Summary

| Test Case | Status | Notes |
|-----------|--------|-------|
| Basic Project Operations | - | Create, list, get, update |
| Milestone Management | - | Add, complete |
| Task Management | - | Add, complete |
| Filtering | - | By tag, by status |
| Statistics | - | Stats command |
| JSON Output | - | JSON format for all commands |
| Error Handling | - | Non-existent resources |
| Example/Skill Commands | - | Help commands |
