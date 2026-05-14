use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Pig (cravings) tracking CLI - record when you have cravings for junk food.

Key features:
- Quick recording of food cravings
- Track what you ate when craving hit
- Tag support for categorization
- Time-based history"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <FOOD>: Record a craving
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-pig"
description: "Records food cravings/junk food binges. Invoke when user wants to track when they have cravings or eat junk food."
---

# i-rs-pig

## Commands

### add
Record a craving.
```bash
i-rs-pig add <FOOD_NAME> [--description] [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-pig list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-pig get <ID>
```

### delete
Delete a record.
```bash
i-rs-pig delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}
