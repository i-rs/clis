use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Towel replacement tracking CLI - record when you replace towels.

Key features:
- Track towel types
- Time-based history
- Tag support for categorization"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <TOWEL_TYPE>: Record towel replacement
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-towel"
description: "Records towel replacements. Invoke when user wants to track when they replace towels."
---

# i-rs-towel

## Commands

### add
Record towel replacement.
```bash
i-rs-towel add <TOWEL_TYPE> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-towel list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-towel get <ID>
```

### delete
Delete a record.
```bash
i-rs-towel delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}