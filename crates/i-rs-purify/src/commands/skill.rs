use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Water purifier filter replacement tracking CLI.

Key features:
- Track filter types
- Time-based history
- Tag support for categorization"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <FILTER_TYPE>: Record filter replacement
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-purify"
description: "Records water purifier filter replacements. Invoke when user wants to track when they replace water purifier filters."
---

# i-rs-purify

## Commands

### add
Record filter replacement.
```bash
i-rs-purify add <FILTER_TYPE> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-purify list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-purify get <ID>
```

### delete
Delete a record.
```bash
i-rs-purify delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}