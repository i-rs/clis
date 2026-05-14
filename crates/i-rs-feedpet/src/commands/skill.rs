use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Pet feeding tracking CLI - record when you feed your pets.

Key features:
- Track different pets and food types
- Record feeding amounts
- Time-based history
- Tag support"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <PET_NAME> <FOOD_TYPE> <AMOUNT>: Record pet feeding
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-feedpet"
description: "Records pet feeding. Invoke when user wants to track when they feed their pets."
---

# i-rs-feedpet

## Commands

### add
Record pet feeding.
```bash
i-rs-feedpet add <PET_NAME> <FOOD_TYPE> <AMOUNT> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-feedpet list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-feedpet get <ID>
```

### delete
Delete a record.
```bash
i-rs-feedpet delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}