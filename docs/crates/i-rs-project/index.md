# i-rs-project

Project management CLI tool for organizing and tracking projects with milestones and tasks.

## Overview

i-rs-project helps you manage multiple projects efficiently. Each project can have milestones and tasks, with support for status tracking, priority levels, and tags.

## Quick Start

```bash
# Create a new project
i-rs-project add my-project --description "My first project"

# List all projects
i-rs-project list

# Add a milestone
i-rs-project milestone add my-project "v1.0" --due-date 2026-06-01

# Add a task
i-rs-project task add my-project "Write documentation"

# Complete a task
i-rs-project task complete my-project "Write documentation"
```

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records

## Features

- **Project Management**: Create, update, delete, and list projects
- **Milestones**: Add milestones with optional due dates
- **Tasks**: Add and track tasks within projects
- **Status Tracking**: active, onhold, completed, cancelled
- **Priority Levels**: low, medium, high, urgent
- **Tags**: Organize projects with custom tags
- **Statistics**: Overview of all projects

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-project

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-project
```

## Data Storage

- macOS: `~/.config/i-rs/project.json`
- Linux: `~/.config/i-rs/project.json`
- Windows: `~\AppData\Roaming\i-rs\project.json`
