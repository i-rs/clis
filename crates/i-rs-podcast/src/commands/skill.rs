use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Podcast and course tracking CLI - manage your audio/video learning content.

Key features:
- Track podcasts and courses with title, author, and duration
- Track listening progress (current position)
- Add notes during listening sessions
- Filter by status (not_started, in_progress, completed)
- Tag support for organization
- Statistics overview with progress percentage

Invoke when: tracking podcast episodes, managing course progress, or recording learning notes."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME>: Add a podcast/course
- list [--status|--tag]: List podcasts
- get <NAME>: Show podcast details
- listen <NAME>: Update listening progress
- update <NAME>: Update podcast info
- delete <NAME>: Delete a podcast
- stats: Show statistics
- example: Show examples

Options:
- --author, -a: Author/host name
- --duration, -d: Total duration in seconds
- --position, -p: Current position in seconds (listen command)
- --status, -s: Status filter
- --tag, -t: Tags (repeatable)
- --remark: Remark lines (repeatable)
- --notes: Note lines (repeatable)

Status:
- not_started: ○
- in_progress: ◐
- completed: ●"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-podcast"
description: "Podcast and course tracking CLI (add/listen/update/stats). Invoke when user needs to track podcast episodes, manage course progress, or record learning notes."
---

# i-rs-podcast

## Commands

### add
Add a podcast or course to collection.
```bash
i-rs-podcast add <NAME> [--author] [--duration] [--tag] [--remark] [--notes]
```

### list
List all podcasts with optional filters.
```bash
i-rs-podcast list [--status STATUS] [--tag TAG]
```

### get
Show detailed podcast information.
```bash
i-rs-podcast get <NAME>
```

### listen
Update listening progress with current position.
```bash
i-rs-podcast listen <NAME> --position SECONDS [--notes]
```

### update
Update podcast information.
```bash
i-rs-podcast update <NAME> [--author] [--duration] [--tag] [--remark] [--notes]
```

### delete
Delete a podcast from collection.
```bash
i-rs-podcast delete <NAME>
```

### stats
Show podcast collection statistics.
```bash
i-rs-podcast stats
```

## Status
- `not_started` (○): Not started
- `in_progress` (◐): Currently listening
- `completed` (●): Finished"#;

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
