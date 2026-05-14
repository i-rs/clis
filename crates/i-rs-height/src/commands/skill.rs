use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Height tracking CLI - track height and weight over time with statistics and charts.

Key features:
- Record height measurements with optional weight
- View history with ASCII charts
- Calculate min/max/average statistics
- Track height change over time
- Set target height goals
- Tag and remark support

Invoke when: tracking growth, monitoring body measurements, or maintaining height history."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <DATE> <HEIGHT>: Add height record
- list [--days] [--chart] [--stats]: List records with optional chart/stats
- get <DATE>: Get specific record details
- delete <DATE>: Delete height record
- set <HEIGHT>: Set target height
- target: Show current target height

Options:
- --days, -d: Show records from last N days
- --chart, -c: Display ASCII chart
- --stats, -s: Show statistics
- --weight, -w: Weight in kg (optional)
- --tag, -t: Tags (repeatable)
- --remark, -r: Remarks (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-height"
description: "Tracks body height (add/list/get/delete). Invoke when user needs to record height measurements, view height history, or display statistics and charts."
---

# i-rs-height

## Commands

### add
Add a height record.
```bash
i-rs-height add <DATE> <HEIGHT> [--weight] [--tag] [--remark]
```

### list
List height records.
```bash
i-rs-height list [--days N] [--chart] [--stats]
```

### get
Get a specific height record.
```bash
i-rs-height get <DATE>
```

### delete
Delete a height record.
```bash
i-rs-height delete <DATE>
```

### set
Set target height.
```bash
i-rs-height set <HEIGHT>
```

### target
Show current target height.
```bash
i-rs-height target
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
