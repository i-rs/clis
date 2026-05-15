---
name: "i-rs-project"
description: "Manages projects with milestones and tasks (add/list/get/update/delete). Invoke when user needs to track projects, manage milestones, or organize tasks."
---

# i-rs-project

Project management CLI tool for organizing and tracking projects with milestones and tasks.

## Storage

- Config: `~/.config/i-rs/project.json`

## Global Flags

- `--json` — Output in JSON format
## Project Status

| Value | Description |
|-------|-------------|
| active | Project in progress (default) |
| onhold | Project paused |
| completed | Project finished |
| cancelled | Project cancelled |

## Priority Levels

| Value | Description |
|-------|-------------|
| low | Low priority |
| medium | Medium priority (default) |
| high | High priority |
| urgent | Urgent priority |

## Commands

### add

Add a new project.

```bash
i-rs-project add <NAME> [OPTIONS]
```

Options:
- `-d, --description <TEXT>` - Project description
- `-s, --status <STATUS>` - Status (active, onhold, completed, cancelled)
- `-p, --priority <PRIORITY>` - Priority (low, medium, high, urgent)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all projects.

```bash
i-rs-project list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag
- `-s, --status <STATUS>` - Filter by status

### get

Get project details.

```bash
i-rs-project get <NAME>
```

### update

Update a project.

```bash
i-rs-project update <NAME>
```

Options:
- `-d, --description <TEXT>` - New description
- `-s, --status <STATUS>` - New status
- `-p, --priority <PRIORITY>` - New priority
- `-t, --tag <TAGS>` - Replace tags
- `-r, --remark <REMARKS>` - Replace remarks

### delete

Delete a project.

```bash
i-rs-project delete <NAME>
```

### milestone

Manage project milestones.

#### milestone add

```bash
i-rs-project milestone add <PROJECT> <MILESTONE>
```

Options:
- `-d, --description <TEXT>` - Milestone description
- `-D, --due-date <DATE>` - Due date (YYYY-MM-DD)

#### milestone complete

```bash
i-rs-project milestone complete <PROJECT> <MILESTONE>
```

### task

Manage project tasks.

#### task add

```bash
i-rs-project task add <PROJECT> <TASK>
```

Options:
- `-d, --description <TEXT>` - Task description
- `-t, --tag <TAG>` - Tags (can be repeated)

#### task complete

```bash
i-rs-project task complete <PROJECT> <TASK>
```

### stats

Show project statistics.

```bash
i-rs-project stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-project data export
i-rs-project data import [FILE]
i-rs-project data clear
```

### example

Show usage examples.

```bash
i-rs-project example
```

### skill

Show skill information.

```bash
i-rs-project skill [summary|content|raw]
```

## Examples

```bash
# Add a new project
i-rs-project add my-project --description "My project" --priority high --tag work [OPTIONS]

# List all projects
i-rs-project list [OPTIONS]

# Filter by tag
i-rs-project list --tag work

# Get project details
i-rs-project get my-project

# Update project
i-rs-project update my-project --status completed

# Add milestone
i-rs-project milestone add my-project "v1.0" --due-date 2026-06-01

# Complete milestone
i-rs-project milestone complete my-project "v1.0"

# Add task
i-rs-project task add my-project "Write docs"

# Complete task
i-rs-project task complete my-project "Write docs"

# View statistics
i-rs-project stats
```
