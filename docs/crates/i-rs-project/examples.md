# i-rs-project Examples

Detailed usage examples for i-rs-project.

## Basic Project Management

### Creating Projects

```bash
# Create a simple project
i-rs-project add my-project

# Create with description
i-rs-project add web-redesign --description "Complete website redesign project"

# Create with priority
i-rs-project add api-migration --priority high

# Create with multiple tags
i-rs-project add mobile-app --tag work --tag mobile --tag urgent

# Create with status
i-rs-project add side-project --status onhold --description "Weekend project"
```

### Listing Projects

```bash
# List all projects
i-rs-project list

# Filter by tag
i-rs-project list --tag work
i-rs-project list --tag frontend

# Filter by status
i-rs-project list --status active
i-rs-project list --status completed

# Combine filters
i-rs-project list --tag work --status active
```

### Viewing Projects

```bash
# Get project details
i-rs-project get my-project

# Get another project
i-rs-project get api-redesign
```

### Updating Projects

```bash
# Update status
i-rs-project update my-project --status completed
i-rs-project update my-project --status onhold

# Update priority
i-rs-project update my-project --priority urgent
i-rs-project update my-project --priority low

# Update description
i-rs-project update my-project --description "Updated project scope"

# Update multiple fields
i-rs-project update my-project --status active --priority high --tag urgent

# Update tags
i-rs-project update my-project --tag work --tag important
```

### Deleting Projects

```bash
# Delete a project
i-rs-project delete my-project
```

## Milestone Management

### Adding Milestones

```bash
# Add basic milestone
i-rs-project milestone add my-project "v1.0"

# Add milestone with due date
i-rs-project milestone add my-project "Beta Release" --due-date 2026-06-01

# Add milestone with description
i-rs-project milestone add my-project "Launch" --description "Product launch date" --due-date 2026-07-15

# Add multiple milestones
i-rs-project milestone add my-project "Design Complete" --due-date 2026-05-01
i-rs-project milestone add my-project "Development Complete" --due-date 2026-06-01
i-rs-project milestone add my-project "QA Complete" --due-date 2026-06-15
i-rs-project milestone add my-project "Production" --due-date 2026-07-01
```

### Completing Milestones

```bash
# Mark milestone as complete
i-rs-project milestone complete my-project "v1.0"

# Complete another milestone
i-rs-project milestone complete my-project "Beta Release"
```

## Task Management

### Adding Tasks

```bash
# Add basic task
i-rs-project task add my-project "Write documentation"

# Add task with description
i-rs-project task add my-project "Review code" --description "Review all pull requests"

# Add task with tags
i-rs-project task add my-project "Update tests" --tag testing --tag important

# Add multiple tasks
i-rs-project task add my-project "Setup CI/CD"
i-rs-project task add my-project "Write unit tests"
i-rs-project task add my-project "Create deployment scripts"
i-rs-project task add my-project "Setup monitoring"
```

### Completing Tasks

```bash
# Mark task as complete
i-rs-project task complete my-project "Write documentation"

# Complete another task
i-rs-project task complete my-project "Review code"
```

## Statistics

### Viewing Stats

```bash
# Show all project statistics
i-rs-project stats
```

## JSON Output

All commands support JSON output with the `--json` flag:

```bash
# List projects in JSON
i-rs-project list --json

# Get project details in JSON
i-rs-project get my-project --json

# Stats in JSON
i-rs-project stats --json
```

## Complete Workflow Example

```bash
# 1. Create a new project
i-rs-project add website-redesign \
  --description "Complete redesign of company website" \
  --priority high \
  --tag work \
  --tag important

# 2. Add milestones
i-rs-project milestone add website-redesign "Design Phase" \
  --description "Complete all designs" \
  --due-date 2026-05-15

i-rs-project milestone add website-redesign "Development Phase" \
  --description "Implement all features" \
  --due-date 2026-06-15

i-rs-project milestone add website-redesign "Launch" \
  --description "Website goes live" \
  --due-date 2026-07-01

# 3. Add tasks
i-rs-project task add website-redesign "Create wireframes" --tag design
i-rs-project task add website-redesign "Design homepage" --tag design
i-rs-project task add website-redesign "Design inner pages" --tag design
i-rs-project task add website-redesign "Implement frontend" --tag development
i-rs-project task add website-redesign "Write content" --tag content
i-rs-project task add website-redesign "SEO optimization" --tag marketing

# 4. Complete some tasks
i-rs-project task complete website-redesign "Create wireframes"
i-rs-project task complete website-redesign "Write content"

# 5. Complete a milestone
i-rs-project milestone complete website-redesign "Design Phase"

# 6. Check progress
i-rs-project get website-redesign

# 7. View statistics
i-rs-project stats

# 8. Update status when done
i-rs-project update website-redesign --status completed
```

## Tag-Based Organization

```bash
# Create projects with different tags
i-rs-project add project-alpha --tag work --tag frontend
i-rs-project add project-beta --tag work --tag backend
i-rs-project add personal-project --tag personal
i-rs-project add urgent-task --tag urgent --tag important

# List by tag
i-rs-project list --tag work
i-rs-project list --tag frontend
i-rs-project list --tag personal

# Update project tags
i-rs-project update project-alpha --tag work --tag frontend --tag react
```
