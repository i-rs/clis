use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Aquarium water change tracking CLI - record when you change aquarium water.

Key features:
- Track tank size optionally
- Time-based history
- Tag support for categorization"#;

const SKILL_CONTENT: &str = r#"Commands:
- add [--tank-size]: Record water change
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-aqua"
description: "Records aquarium water changes. Invoke when user wants to track when they change fish tank water."
---

# i-rs-aqua

## Commands

### add
Record water change.
```bash
i-rs-aqua add [--tank-size LITERS] [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-aqua list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-aqua get <ID>
```

### delete
Delete a record.
```bash
i-rs-aqua delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}