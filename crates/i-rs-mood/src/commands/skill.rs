use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Mood tracking CLI - track daily emotional states with visual calendar.

Key features:
- Record daily mood with 5 levels (happy/good/neutral/bad/terrible)
- View mood history in table or calendar format
- Statistics showing best/worst/average mood
- Tag and content support for context
- Visual emoji calendar for quick overview

Invoke when: tracking emotional well-being, recording daily moods, or reviewing mood patterns."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <DATE> <MOOD>: Add mood record
- list [--days] [--calendar]: List records with optional calendar
- update <DATE>: Update mood record
- delete <DATE>: Delete mood record

Mood Values: happy, good, neutral, bad, terrible

Options:
- --tag, -t: Tags (repeatable)
- --content, -c: Content lines (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-mood"
description: "Tracks mood (add/list/update/delete). Invoke when user needs to record daily emotional states, view mood history, or display mood calendar."
---

# i-rs-mood

## Commands

### add
Add a mood record.
```bash
i-rs-mood add <DATE> <MOOD> [--tag] [--content]
```

### list
List mood records.
```bash
i-rs-mood list [--days N] [--calendar]
```

### update
Update a mood record.
```bash
i-rs-mood update <DATE> [--mood] [--tag] [--content]
```

### delete
Delete a mood record.
```bash
i-rs-mood delete <DATE>
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