# i-rs-todo Usage

## install

```bash
npm install -g @i-rs/i-rs-todo
# or
brew install i-rs/homebrew-tap/i-rs-todo
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

| Input | Emoji | Label |
|-------|-------|-------|
| high, h, 3 | 🔴 | High |
| medium, med, m, 2 | 🟡 | Medium |
| low, l, 1 | 🟢 | Low |

## Examples

### Basic Usage

```bash
# Create a simple todo
i-rs-todo add task-1 --title "My first task"

# Create with priority
i-rs-todo add task-2 --title "Important task" --priority high

# Create with tags
i-rs-todo add task-3 --title "Shopping" --tag shopping --tag urgent
```

### List Todos

```bash
# List all
i-rs-todo list

# List pending only
i-rs-todo list --pending

# List done only
i-rs-todo list --done

# Filter by tag
i-rs-todo list --tag work
```

## Data Storage

- macOS: `~/.config/i-rs/todos.json`
- Linux: `~/.config/i-rs/todos.json`
- Windows: `~\AppData\Roaming\i-rs\todo.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-todo list
```
