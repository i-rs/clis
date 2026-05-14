use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Step tracking CLI - record daily step counts.

Key features:
- Track daily steps
- Optional distance tracking
- Summary statistics (total, average)
- Date-based records"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <STEPS> <DATE>: Add step record
- list: List all records with summary
- get <DATE>: Get record for specific date
- update <DATE>: Update record
- delete <DATE>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-step"
description: "Records daily step counts. Invoke when user wants to track walking/running steps."
---

# i-rs-step

## Commands

### add
Add step record.
```bash
i-rs-step add <STEPS> <DATE> [--distance KM] [--tag] [--remark]
```

### list
List all records with summary.
```bash
i-rs-step list
```

### get
Get record for date.
```bash
i-rs-step get <DATE>
```

### update
Update record.
```bash
i-rs-step update <DATE> [--steps] [--distance] [--tag] [--remark]
```

### delete
Delete record.
```bash
i-rs-step delete <DATE>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}
