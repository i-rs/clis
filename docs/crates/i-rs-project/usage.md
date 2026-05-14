# i-rs-project Usage

Detailed command reference for i-rs-project.

## Global Options

- `--json, -j` - Output in JSON format

## add

Add a new project.

```bash
i-rs-project add <NAME> [OPTIONS]
```

Options:
- `-d, --description <TEXT>` - Project description
- `-s, --status <STATUS>` - Project status (active, onhold, completed, cancelled)
- `-p, --priority <PRIORITY>` - Priority level (low, medium, high, urgent)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

Examples:
```bash
# Basic project
i-rs-project add my-project

# With description and priority
i-rs-project add api-redesign --description "Redesign the API" --priority high

# With tags
i-rs-project add web-app --tag work --tag frontend --priority urgent
```

## list

List all projects.

```bash
i-rs-project list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag
- `-s, --status <STATUS>` - Filter by status

Examples:
```bash
# List all projects
i-rs-project list

# Filter by tag
i-rs-project list --tag work

# Filter by status
i-rs-project list --status active
```

## get

Get project details.

```bash
i-rs-project get <NAME>
```

Example:
```bash
i-rs-project get my-project
```

## update

Update an existing project.

```bash
i-rs-project update <NAME> [OPTIONS]
```

Options:
- `-d, --description <TEXT>` - New description
- `-s, --status <STATUS>` - New status
- `-p, --priority <PRIORITY>` - New priority
- `-t, --tag <TAGS>` - Replace tags (can be repeated)
- `-r, --remark <REMARKS>` - Replace remarks (can be repeated)

Examples:
```bash
# Update status
i-rs-project update my-project --status completed

# Update priority
i-rs-project update my-project --priority urgent

# Update multiple fields
i-rs-project update my-project --status onhold --description "Paused for review"
```

## delete

Delete a project.

```bash
i-rs-project delete <NAME>
```

Example:
```bash
i-rs-project delete my-project
```

## milestone

Manage project milestones.

### milestone add

Add a milestone to a project.

```bash
i-rs-project milestone add <PROJECT> <MILESTONE> [OPTIONS]
```

Options:
- `-d, --description <TEXT>` - Milestone description
- `-D, --due-date <DATE>` - Due date (YYYY-MM-DD format)

Examples:
```bash
# Basic milestone
i-rs-project milestone add my-project "v1.0"

# With due date
i-rs-project milestone add my-project "Beta Release" --due-date 2026-06-01

# With description
i-rs-project milestone add my-project "Launch" --description "Product launch" --due-date 2026-07-01
```

### milestone complete

Mark a milestone as completed.

```bash
i-rs-project milestone complete <PROJECT> <MILESTONE>
```

Example:
```bash
i-rs-project milestone complete my-project "v1.0"
```

## task

Manage project tasks.

### task add

Add a task to a project.

```bash
i-rs-project task add <PROJECT> <TASK> [OPTIONS]
```

Options:
- `-d, --description <TEXT>` - Task description
- `-t, --tag <TAG>` - Tags (can be repeated)

Examples:
```bash
# Basic task
i-rs-project task add my-project "Write docs"

# With description and tags
i-rs-project task add my-project "Review PR" --description "Review pull request" --tag code-review
```

### task complete

Mark a task as completed.

```bash
i-rs-project task complete <PROJECT> <TASK>
```

Example:
```bash
i-rs-project task complete my-project "Write docs"
```

## stats

Show project statistics.

```bash
i-rs-project stats
```

Example:
```bash
i-rs-project stats
```

## example

Show usage examples.

```bash
i-rs-project example
```

## skill

View AI skill documentation.

```bash
i-rs-project skill [SUB_COMMAND]
```

Subcommands:
- `summary` - Show skill summary
- `content` - Show full skill content
- `raw` - Show raw skill document
