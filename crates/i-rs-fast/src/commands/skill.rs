use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Fasting tracking CLI - record fasting sessions.

Key features:
- Track fasting start time
- Set target fasting hours
- Auto-calculate actual duration when ended
- Tag support"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <TARGET_HOURS>: Start a fast
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-fast"
description: "Records fasting sessions. Invoke when user wants to track fasting."
---

# i-rs-fast

## Commands

### add
Start a fast.
```bash
i-rs-fast add <TARGET_HOURS> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-fast list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-fast get <ID>
```

### delete
Delete a record.
```bash
i-rs-fast delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}