use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Toothbrush replacement tracking CLI - record when you replace toothbrushes.

Key features:
- Track toothbrush types
- Time-based history
- Tag support for categorization"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <BRUSH_TYPE>: Record toothbrush replacement
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-toothbrush"
description: "Records toothbrush replacements. Invoke when user wants to track when they replace toothbrushes."
---

# i-rs-toothbrush

## Commands

### add
Record toothbrush replacement.
```bash
i-rs-toothbrush add <BRUSH_TYPE> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-toothbrush list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-toothbrush get <ID>
```

### delete
Delete a record.
```bash
i-rs-toothbrush delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}