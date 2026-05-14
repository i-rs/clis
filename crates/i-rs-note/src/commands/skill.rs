use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Note management CLI - store and organize text notes with titles and tags.

Key features:
- Create notes with optional titles
- Organize with tags
- Multi-line content support
- Timestamps for created/updated

Invoke when: creating notes, retrieving saved information, or organizing personal knowledge."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME>: Add a new note with title, tags, and content
- list [--tag]: List all notes or filter by tag
- get <NAME>: Get note details with full content
- update <NAME>: Update note info
- delete <NAME>: Delete note

Options:
- --title, -t: Note title (optional)
- --tag, -g: Tags (repeatable)
- --content, -c: Content lines (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-note"
description: "Manages notes (add/list/get/update/delete). Invoke when user needs to create, retrieve, or organize text notes."
---

# i-rs-note

## Commands

### add
Add a new note.
```bash
i-rs-note add <NAME> [--title] [--tag] [--content]
```

### list
List all notes.
```bash
i-rs-note list [--tag TAG]
```

### get
Get note details.
```bash
i-rs-note get <NAME>
```

### update
Update a note.
```bash
i-rs-note update <NAME> [--title] [--tag] [--content]
```

### delete
Delete a note.
```bash
i-rs-note delete <NAME>
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