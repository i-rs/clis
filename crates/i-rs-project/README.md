# i-rs-project

Project management CLI tool for organizing and tracking projects with milestones and tasks.

## Features

- Create and manage multiple projects
- Track project status (active, on hold, completed, cancelled)
- Set priority levels (low, medium, high, urgent)
- Milestone management with due dates
- Task management within projects
- Tag support for organization
- Statistics overview
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-project
# or
brew install i-rs/homebrew-tap/i-rs-project
```

## Quick Start

```bash
# Add a new project
i-rs-project add my-project -d "Project description"

# Add with priority and tags
i-rs-project add api-redesign --description "API redesign" --priority high --tag work

# List all projects
i-rs-project list

# Get project details
i-rs-project get my-project

# Add a milestone
i-rs-project milestone add my-project "v1.0" --due-date 2026-06-01

# Add a task
i-rs-project task add my-project "Write docs"

# Complete a task
i-rs-project task complete my-project "Write docs"

# View statistics
i-rs-project stats
```

## Project Status

| Status | Description |
|--------|-------------|
| active | Project is in progress |
| onhold | Project is paused |
| completed | Project is finished |
| cancelled | Project was cancelled |

## Priority Levels

| Priority | Description |
|----------|-------------|
| low | Low priority |
| medium | Medium priority (default) |
| high | High priority |
| urgent | Urgent priority |

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/project.json`
- Linux: `~/.config/i-rs/project.json`
- Windows: `~\AppData\Roaming\i-rs\project.json`

## License

MIT OR Apache-2.0
