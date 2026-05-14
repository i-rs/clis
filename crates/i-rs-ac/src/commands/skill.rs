use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"AC cleaning tracking CLI - record when you clean air conditioners.

Key features:
- Track different AC locations
- Time-based history
- Tag support for categorization"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <LOCATION>: Record AC cleaning
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-ac"
description: "Records AC cleaning. Invoke when user wants to track when they clean air conditioners."
---

# i-rs-ac

## Commands

### add
Record AC cleaning.
```bash
i-rs-ac add <LOCATION> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-ac list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-ac get <ID>
```

### delete
Delete a record.
```bash
i-rs-ac delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}