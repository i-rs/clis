use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Pet bath tracking CLI - record when you bathe your pets.

Key features:
- Track pet bathing events
- Time-based history
- Tag support for categorization"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <PET_NAME>: Record pet bath
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-petbath"
description: "Records pet baths. Invoke when user wants to track when they bathe their pets."
---

# i-rs-petbath

## Commands

### add
Record pet bath.
```bash
i-rs-petbath add <PET_NAME> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-petbath list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-petbath get <ID>
```

### delete
Delete a record.
```bash
i-rs-petbath delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}