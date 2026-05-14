---
name: "i-rs-todo"
description: "Manages todos (add/list/get/update/delete/done). Invoke when user needs to track tasks, manage to-do lists, or organize work items."
---

# i-rs-todo

Todo management CLI tool for tracking tasks and to-do items.

## Storage

- Config: `~/.config/i-rs/todos.json`

## Priority Levels

| Input | Emoji | Label |
|-------|-------|-------|
| high, h, 3 | 🔴 | High |
| medium, med, m, 2 | 🟡 | Medium |
| low, l, 1 | 🟢 | Low |

## Commands

### add

Add a new todo.

```bash
i-rs-todo add <NAME>
```

Options:
- `-t, --title <TITLE>` - Todo title
- `-p, --priority <PRIORITY>` - Priority (high/medium/low)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-c, --content <CONTENT>` - Content/notes (can be repeated)

### list

List todos.

```bash
i-rs-todo list
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
i-rs-todo update <NAME>
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

## Examples

```bash
# Add a todo
i-rs-todo add task-1 --title "Complete report" --priority high --tag work

# List all
i-rs-todo list

# List pending only
i-rs-todo list --pending

# Mark as done
i-rs-todo done task-1

# Get details
i-rs-todo get task-1

# Update
i-rs-todo update task-1 --priority low

# Delete
i-rs-todo delete task-1
```
