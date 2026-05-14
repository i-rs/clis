# i-rs-todo

Todo management CLI tool for tracking tasks and to-do items.

## Overview

i-rs-todo helps you manage your tasks efficiently. Create todos with priorities, mark them as done, and organize with tags.

## Quick Start

```bash
# Add a new todo
i-rs-todo add work-1 --title "Complete report" --priority high --tag work

# Add a todo with content
i-rs-todo add buy-milk --title "Buy milk" --priority medium --content "Get 2% milk" --tag shopping

# List all todos
i-rs-todo list

# List pending todos only
i-rs-todo list --pending

# Mark todo as done
i-rs-todo done work-1

# Get todo details
i-rs-todo get work-1

# Update todo
i-rs-todo update work-1 --priority low --content "New content"

# Delete todo
i-rs-todo delete work-1
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-todo

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-todo
```

## Data Storage

- macOS: `~/.config/i-rs/todos.json`
- Linux: `~/.config/i-rs/todos.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Priority Levels**: High (🔴), Medium (🟡), Low (🟢)
- **Tags**: Organize todos with multiple tags
- **Content**: Add detailed descriptions
- **Status Tracking**: Mark todos as done/pending
- **Filtering**: View all, pending, or done todos

## Priority Levels

| Level | Input | Emoji | Description |
|-------|-------|-------|-------------|
| High | high, h, 3 | 🔴 | High priority |
| Medium | medium, med, m, 2 | 🟡 | Medium priority (default) |
| Low | low, l, 1 | 🟢 | Low priority |

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records
