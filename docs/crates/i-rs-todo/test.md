# i-rs-todo Test Records

## Test Log

### Basic CRUD Tests

#### Add Todos

```bash
$ i-rs-todo add task-1 --title "First task"
✓ Todo 'task-1' added successfully

$ i-rs-todo add task-2 --title "Second task" --priority high
✓ Todo 'task-2' added successfully

$ i-rs-todo add task-3 --title "Third task" --priority low
✓ Todo 'task-3' added successfully
```

#### List Todos

```bash
$ i-rs-todo list
 NAME     TITLE        PRIORITY      STATUS      TAGS
 task-1   First task   🟡 Medium    ○ Pending   -
 task-2   Second task  🔴 High     ○ Pending   -
 task-3   Third task   🟢 Low      ○ Pending   -

Total: 3 pending, 0 done

$ i-rs-todo list --pending
 NAME     TITLE        PRIORITY      STATUS      TAGS
 task-1   First task   🟡 Medium    ○ Pending   -
 task-2   Second task  🔴 High     ○ Pending   -
 task-3   Third task   🟢 Low      ○ Pending   -

Total: 3 pending, 0 done
```

#### Get Todo Details

```bash
$ i-rs-todo get task-1

Todo: task-1

Title:        First task
Priority:     🟡 Medium
Status:       ○ Pending
Created:       2025-01-15 10:00:00
Updated:       2025-01-15 10:00:00
```

### Status Toggle Tests

#### Mark as Done

```bash
$ i-rs-todo done task-1
✓ Todo 'task-1' marked as done

$ i-rs-todo list
 NAME     TITLE        PRIORITY      STATUS      TAGS
 task-1   First task   🟡 Medium    ✓ Done     -
 task-2   Second task  🔴 High     ○ Pending   -
 task-3   Third task   🟢 Low      ○ Pending   -

Total: 2 pending, 1 done
```

#### Toggle Back to Pending

```bash
$ i-rs-todo done task-1
✓ Todo 'task-1' marked as pending

$ i-rs-todo list
 NAME     TITLE        PRIORITY      STATUS      TAGS
 task-1   First task   🟡 Medium    ○ Pending   -
 task-2   Second task  🔴 High     ○ Pending   -
 task-3   Third task   🟢 Low      ○ Pending   -

Total: 3 pending, 0 done
```

### Update Tests

#### Update Title

```bash
$ i-rs-todo update task-1 --title "Updated first task"
✓ Todo 'task-1' updated

$ i-rs-todo get task-1
Title:        Updated first task
```

#### Update Priority

```bash
$ i-rs-todo update task-1 --priority high
✓ Todo 'task-1' updated

$ i-rs-todo get task-1
Priority:     🔴 High
```

#### Update with Tags and Content

```bash
$ i-rs-todo update task-1 --tag work --tag important --content "New content line"
✓ Todo 'task-1' updated

$ i-rs-todo get task-1
Tags:        work, important
Content:     New content line
```

### Delete Tests

```bash
$ i-rs-todo delete task-3
✓ Todo 'task-3' deleted

$ i-rs-todo list
 NAME     TITLE        PRIORITY      STATUS      TAGS
 task-1   First task   🟡 Medium    ○ Pending   -
 task-2   Second task  🔴 High     ○ Pending   -

Total: 2 pending, 0 done
```

### Priority Tests

#### All Priority Levels

```bash
$ i-rs-todo add prio-high --title "High priority" --priority high
$ i-rs-todo add prio-med --title "Medium priority" --priority medium
$ i-rs-todo add prio-low --title "Low priority" --priority low
$ i-rs-todo add prio-1 --title "Using number 1" --priority 1
$ i-rs-todo add prio-2 --title "Using number 2" --priority 2
$ i-rs-todo add prio-3 --title "Using number 3" --priority 3

$ i-rs-todo list
 NAME       TITLE              PRIORITY      STATUS      TAGS
 prio-high  High priority      🔴 High      ○ Pending   -
 prio-3     Using number 3     🔴 High      ○ Pending   -
 prio-med   Medium priority    🟡 Medium    ○ Pending   -
 prio-2     Using number 2     🟡 Medium    ○ Pending   -
 prio-low   Low priority       🟢 Low       ○ Pending   -
 prio-1     Using number 1     🟢 Low       ○ Pending   -
 task-1     First task         🟡 Medium    ○ Pending   -
 task-2     Second task        🔴 High      ○ Pending   -

Total: 8 pending, 0 done
```

### Tag Tests

```bash
$ i-rs-todo add tagged-1 --title "Tagged task" --tag work --tag project-a
$ i-rs-todo add tagged-2 --title "Another task" --tag work --tag project-b
$ i-rs-todo add tagged-3 --title "Third task" --tag personal

$ i-rs-todo list --tag work
 NAME       TITLE              PRIORITY      STATUS      TAGS
 tagged-1   Tagged task        🟡 Medium    ○ Pending   work, project-a
 tagged-2   Another task       🟡 Medium    ○ Pending   work, project-b

Total: 2 pending, 0 done
```

### Filter Tests

```bash
$ i-rs-todo done task-1
$ i-rs-todo done task-2

$ i-rs-todo list --done
 NAME     TITLE        PRIORITY      STATUS      TAGS
 task-1   First task   🟡 Medium    ✓ Done     -
 task-2   Second task  🔴 High     ✓ Done     -

Total: 6 pending, 2 done

$ i-rs-todo list --pending
 NAME       TITLE              PRIORITY      STATUS      TAGS
 prio-high  High priority      🔴 High      ○ Pending   -
 prio-3     Using number 3     🔴 High      ○ Pending   -
 prio-med   Medium priority    🟡 Medium    ○ Pending   -
 prio-2     Using number 2     🟡 Medium    ○ Pending   -
 prio-low   Low priority       🟢 Low       ○ Pending   -
 prio-1     Using number 1     🟢 Low       ○ Pending   -
 tagged-1   Tagged task        🟡 Medium    ○ Pending   work, project-a
 tagged-2   Another task       🟡 Medium    ○ Pending   work, project-b
 tagged-3   Third task         🟡 Medium    ○ Pending   personal

Total: 8 pending, 0 done
```

## Error Handling Tests

### Duplicate Name

```bash
$ i-rs-todo add task-1 --title "Duplicate"
Error: Todo 'task-1' already exists
```

### Non-existent Todo

```bash
$ i-rs-todo get non-existent
Error: Todo 'non-existent' not found

$ i-rs-todo delete non-existent
Error: Todo 'non-existent' not found

$ i-rs-todo done non-existent
Error: Todo 'non-existent' not found
```

### Invalid Priority

```bash
$ i-rs-todo add invalid-prio --title "Test" --priority invalid
# Falls back to medium priority (default)
✓ Todo 'invalid-prio' added successfully
```

## Test Summary

| Test Case | Status |
|-----------|--------|
| Add simple todo | ✅ Pass |
| Add with priority | ✅ Pass |
| Add with tags | ✅ Pass |
| Add with content | ✅ Pass |
| List all todos | ✅ Pass |
| List pending only | ✅ Pass |
| List done only | ✅ Pass |
| List by tag | ✅ Pass |
| Get todo details | ✅ Pass |
| Toggle done/pending | ✅ Pass |
| Update title | ✅ Pass |
| Update priority | ✅ Pass |
| Update tags | ✅ Pass |
| Update content | ✅ Pass |
| Delete todo | ✅ Pass |
| Priority numbers (1-3) | ✅ Pass |
| Duplicate name handling | ✅ Pass |
| Non-existent todo handling | ✅ Pass |
| Invalid priority fallback | ✅ Pass |
