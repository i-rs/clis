use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Bed item replacement tracking CLI - record when you replace mattress, pillows, etc.

Key features:
- Track bed item types (mattress, pillow, etc.)
- Time-based history
- Tag support for categorization"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <ITEM_TYPE>: Record bed item replacement
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-bed"
description: "Records bed item replacements. Invoke when user wants to track when they replace mattress, pillows, etc."
---

# i-rs-bed

## Commands

### add
Record bed item replacement.
```bash
i-rs-bed add <ITEM_TYPE> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-bed list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-bed get <ID>
```

### delete
Delete a record.
```bash
i-rs-bed delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}