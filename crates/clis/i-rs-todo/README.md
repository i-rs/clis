# i-rs-todo

Todo management CLI tool for tracking tasks and to-do items.

## Install

```bash
npm install -g @i-rs/i-rs-todo
# or
brew install i-rs/homebrew-tap/i-rs-todo
```

## Features

- **Priority Levels**: High (🔴), Medium (🟡), Low (🟢)
- **Tags**: Organize todos with multiple tags
- **Content**: Add detailed descriptions
- **Status Tracking**: Mark todos as done/pending
- **Filtering**: View all, pending, or done todos

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

## Commands

### add

Add a new todo.

```bash
i-rs-todo add <NAME> [OPTIONS]
```

Options:
- `-t, --title <TITLE>` - Todo title
- `-p, --priority <PRIORITY>` - Priority (high/medium/low)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-c, --content <CONTENT>` - Content/notes (can be repeated)

### list

List todos.

```bash
i-rs-todo list [OPTIONS]
```

Options:
- `--pending` - Show only pending todos
- `--done` - Show only done todos
- `-t, --tag <TAG>` - Filter by tag

### get

Get todo details.

```bash
i-rs-todo get <NAME>
```

### done

Toggle todo status (done/pending).

```bash
i-rs-todo done <NAME>
```

### update

Update a todo.

```bash
i-rs-todo update <NAME> [OPTIONS]
```

Options:
- `-t, --title <TITLE>` - New title
- `-p, --priority <PRIORITY>` - New priority
- `-t, --tag <TAG>` - New tags
- `-c, --content <CONTENT>` - New content

### delete

Delete a todo.

```bash
i-rs-todo delete <NAME>
```

## Priority Levels

| Level | Input | Emoji | Description |
|-------|-------|-------|-------------|
| High | high, h, 3 | 🔴 | High priority |
| Medium | medium, med, m, 2 | 🟡 | Medium priority (default) |
| Low | low, l, 1 | 🟢 | Low priority |

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/todos.json`
- Linux: `~/.config/i-rs/todos.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-todo list
```

## License

MIT OR Apache-2.0
