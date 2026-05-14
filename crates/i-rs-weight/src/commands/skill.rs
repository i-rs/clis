use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Weight tracking CLI - track body weight over time with statistics and charts.

Key features:
- Record daily weight measurements
- View history with ASCII charts
- Calculate min/max/average statistics
- Track weight change over time
- Optional remarks for context

Invoke when: tracking fitness progress, monitoring weight changes, or maintaining weight history."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <DATE> <WEIGHT>: Add weight record
- list [--days] [--chart] [--stats]: List records with optional chart/stats
- update <DATE>: Update weight record
- delete <DATE>: Delete weight record

Options:
- --days, -d: Show records from last N days
- --chart, -c: Display ASCII chart
- --stats, -s: Show statistics
- --remark, -r: Remarks (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-weight"
description: "Tracks body weight (add/list/update/delete). Invoke when user needs to record weight measurements, view weight history, or display statistics and charts."
---

# i-rs-weight

## Commands

### add
Add a weight record.
```bash
i-rs-weight add <DATE> <WEIGHT> [--remark]
```

### list
List weight records.
```bash
i-rs-weight list [--days N] [--chart] [--stats]
```

### update
Update a weight record.
```bash
i-rs-weight update <DATE> [--weight] [--remark]
```

### delete
Delete a weight record.
```bash
i-rs-weight delete <DATE>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => {
            println!("{}", SKILL_SUMMARY);
        }
        Some(SkillCommand::Content) => {
            println!("{}", SKILL_CONTENT);
        }
        Some(SkillCommand::Raw) | None => {
            println!("{}", SKILL_RAW);
        }
    }
}