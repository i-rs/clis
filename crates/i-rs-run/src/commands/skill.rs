use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Running record CLI - track running activities with distance, pace, duration, and heart rate.

Key features:
- Record running activities with detailed metrics
- Track heart rate and weather conditions
- View cumulative statistics (total distance, duration, average pace)
- Manage running plans with schedules
- Tag support for categorization

Invoke when: tracking running progress, planning training schedules, or monitoring fitness activities."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <DATE> <DISTANCE> <DURATION>: Add run record
- list [--json]: List all run records
- get <ID>: Get run record details
- delete <ID>: Delete run record
- stats: Show cumulative statistics
- plan-add <NAME> <TARGET> <PACE>: Add run plan
- plan-list [--json]: List all plans
- plan-get <ID>: Get plan details
- plan-delete <ID>: Delete plan

Options:
- --heart-rate, -hr: Heart rate (bpm)
- --weather, -w: Weather conditions
- --tags, -t: Tags (repeatable)
- --remark, -r: Remarks (repeatable)
- --schedule, -s: Schedule days (0-6 for Sun-Sat)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-run"
description: "Running record tracker (add/list/get/delete/stats/plan). Invoke when user needs to record running activities, view statistics, or manage training plans."
---

# i-rs-run

## Commands

### add
Add a running record.
```bash
i-rs-run add <DATE> <DISTANCE> <DURATION> [--heart-rate] [--weather] [--tags] [--remark]
```

### list
List all run records.
```bash
i-rs-run list [--json]
```

### get
Get run record details.
```bash
i-rs-run get <ID> [--json]
```

### delete
Delete a run record.
```bash
i-rs-run delete <ID>
```

### stats
Show cumulative statistics.
```bash
i-rs-run stats
```

### plan-add
Add a running plan.
```bash
i-rs-run plan-add <NAME> <TARGET> <PACE> [--schedule] [--tags] [--remark]
```

### plan-list
List all running plans.
```bash
i-rs-run plan-list [--json]
```

### plan-get
Get plan details.
```bash
i-rs-run plan-get <ID> [--json]
```

### plan-delete
Delete a running plan.
```bash
i-rs-run plan-delete <ID>
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
