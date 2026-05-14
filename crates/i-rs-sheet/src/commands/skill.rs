use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Sheet change tracking CLI - record when you change bed sheets.

Key features:
- Track different sheet types (bedsheet, pillowcase, etc.)
- Time-based history
- Tag support for categorization"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <SHEET_TYPE>: Record sheet change
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-sheet"
description: "Records sheet changes. Invoke when user wants to track when they change bed sheets."
---

# i-rs-sheet

## Commands

### add
Record sheet change.
```bash
i-rs-sheet add <SHEET_TYPE> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-sheet list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-sheet get <ID>
```

### delete
Delete a record.
```bash
i-rs-sheet delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}