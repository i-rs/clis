use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Dog walking tracking CLI - record when you walk your dog.

Key features:
- Track walking duration
- Different dogs support
- Time-based history
- Tag support"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <DOG_NAME> <DURATION_MINUTES>: Record dog walk
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-walkdog"
description: "Records dog walks. Invoke when user wants to track when they walk their dog."
---

# i-rs-walkdog

## Commands

### add
Record dog walk.
```bash
i-rs-walkdog add <DOG_NAME> <DURATION_MINUTES> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-walkdog list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-walkdog get <ID>
```

### delete
Delete a record.
```bash
i-rs-walkdog delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}