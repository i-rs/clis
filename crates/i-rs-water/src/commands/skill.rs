use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Water intake tracking CLI - record how much water you drink.

Key features:
- Quick recording of water intake in ml
- Daily intake summary
- Tag support for categorization
- Time-based history"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <AMOUNT_ML>: Record water intake in milliliters
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-water"
description: "Records water intake. Invoke when user wants to track how much water they drink."
---

# i-rs-water

## Commands

### add
Record water intake.
```bash
i-rs-water add <AMOUNT_ML> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-water list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-water get <ID>
```

### delete
Delete a record.
```bash
i-rs-water delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}