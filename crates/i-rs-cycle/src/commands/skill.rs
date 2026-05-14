use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Menstrual cycle tracking CLI - record period, spotting, and symptoms.

Key features:
- Track different event types (period, spotting, ovulation, etc.)
- Record symptoms
- Date-based history
- Tag support"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <DATE> <EVENT_TYPE> [--symptom] [--tag] [--remark]: Record cycle event
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-cycle"
description: "Records menstrual cycle events. Invoke when user wants to track their period or related symptoms."
---

# i-rs-cycle

## Commands

### add
Record a cycle event.
```bash
i-rs-cycle add <DATE> <EVENT_TYPE> [--symptom] [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-cycle list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-cycle get <ID>
```

### delete
Delete a record.
```bash
i-rs-cycle delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}