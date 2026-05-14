use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Todo management CLI - track tasks and to-do items with priorities.

Key features:
- Create todos with title, priority, and tags
- Mark todos as done
- Filter by pending/done status or tags
- Priority levels (high/medium/low) with visual indicators
- Multi-line content support

Invoke when: managing task lists, tracking todo items, or organizing work."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME>: Add a new todo
- list [--pending] [--done] [--tag]: List todos with filters
- get <NAME>: Get todo details
- done <NAME>: Mark todo as done
- update <NAME>: Update todo info
- delete <NAME>: Delete todo

Priority Values: high, medium, low

Options:
- --title, -t: Todo title
- --priority, -p: Priority level
- --tag, -g: Tags (repeatable)
- --content, -c: Content lines (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-todo"
description: "Manages todos (add/list/get/update/delete/done). Invoke when user needs to track tasks, manage todo items, or organize work with priorities."
---

# i-rs-todo

## Commands

### add
Add a new todo.
```bash
i-rs-todo add <NAME> [--title] [--priority] [--tag] [--content]
```

### list
List todos.
```bash
i-rs-todo list [--pending] [--done] [--tag TAG]
```

### get
Get todo details.
```bash
i-rs-todo get <NAME>
```

### done
Mark todo as done.
```bash
i-rs-todo done <NAME>
```

### update
Update a todo.
```bash
i-rs-todo update <NAME> [--title] [--priority] [--tag] [--content]
```

### delete
Delete a todo.
```bash
i-rs-todo delete <NAME>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => {
            println!("{}", SKILL_SUMMARY);
        }
        Some(SkillCommand::Content) => {
            println!("{}", SKILL_CONTENT);
        }
        Some(SkillCommand::Raw) | None => {
            println!("{}", SKILL_RAW);
        }
    }
}