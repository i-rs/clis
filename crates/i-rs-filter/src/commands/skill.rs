use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Appliance filter cleaning tracking CLI - record when you clean appliance filters.

Key features:
- Track different appliances and filter types
- Time-based history
- Tag support for categorization"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <APPLIANCE_NAME> <FILTER_TYPE>: Record filter cleaning
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-filter"
description: "Records appliance filter cleaning. Invoke when user wants to track when they clean appliance filters."
---

# i-rs-filter

## Commands

### add
Record filter cleaning.
```bash
i-rs-filter add <APPLIANCE_NAME> <FILTER_TYPE> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-filter list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-filter get <ID>
```

### delete
Delete a record.
```bash
i-rs-filter delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}