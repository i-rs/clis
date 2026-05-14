use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Sitting duration tracking CLI - record how long you've been sitting.

Key features:
- Track sitting duration in minutes
- Auto-calculate start/end times
- Tag support for categorization
- Time-based history"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <DURATION_MINUTES>: Record sitting duration
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-sit"
description: "Records sitting duration. Invoke when user wants to track how long they've been sitting."
---

# i-rs-sit

## Commands

### add
Record sitting duration.
```bash
i-rs-sit add <DURATION_MINUTES> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-sit list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-sit get <ID>
```

### delete
Delete a record.
```bash
i-rs-sit delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}