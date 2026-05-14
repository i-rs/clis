use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Duration/tick tracking CLI - record time spent on activities.

Key features:
- Track activity durations
- Focus time tracking
- Task duration tracking
- Summary statistics"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <TASK> <SECONDS>: Record duration for a task
- list [--tag]: List all entries with summary
- get <ID>: Get entry details
- delete <ID>: Delete entry"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-tick"
description: "Records duration/time spent on activities. Invoke when user wants to track time spent on tasks, focus sessions, or any timed activities."
---

# i-rs-tick

## Commands

### add
Record a duration.
```bash
i-rs-tick add <TASK> <SECONDS> [--started-at DATETIME] [--description] [--tag] [--remark]
```

### list
List all entries with summary.
```bash
i-rs-tick list [--tag TAG]
```

### get
Get entry details.
```bash
i-rs-tick get <ID>
```

### delete
Delete an entry.
```bash
i-rs-tick delete <ID>
```

## Usage Examples
- Track 25min focus session: `i-rs-tick add "Deep Work" 1500`
- Track 2h coding: `i-rs-tick add "Coding" 7200`
- Track with start time: `i-rs-tick add "Meeting" 3600 --started-at "2024-01-15 14:00"`"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}
