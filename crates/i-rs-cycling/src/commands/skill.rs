use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Cycling record tracking CLI - track cycling activities with distance, duration, speed, and elevation data.

Key features:
- Record cycling activities with date, distance, duration
- Track elevation gain for hill climbing
- Add route descriptions and tags
- Calculate average speed automatically
- View cumulative statistics

Invoke when: tracking cycling workouts, monitoring fitness progress, or maintaining cycling history."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <DATE> <DISTANCE> <DURATION>: Add cycling record
- list [--tag]: List records with optional tag filter
- get <ID|DATE>: View record details
- update <ID|DATE>: Update record
- delete <ID|DATE>: Delete record
- stats: View cumulative statistics

Options:
- --elevation, -e: Elevation gain in meters
- --route, -r: Route description
- --tag, -t: Tags (repeatable)
- --remark, -m: Remarks (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-cycling"
description: "Tracks cycling activities (add/list/get/update/delete). Invoke when user needs to record cycling workouts, view cycling history, or display statistics."
---

# i-rs-cycling

## Storage

- Config: `~/.config/i-rs/cycling.json`

## Commands

### add
Add a cycling record.
```bash
i-rs-cycling add <DATE> <DISTANCE> <DURATION> [--elevation] [--route] [--tag] [--remark]
```

### list
List cycling records.
```bash
i-rs-cycling list [--tag TAG]
```

### get
View record details.
```bash
i-rs-cycling get <ID|DATE>
```

### update
Update a cycling record.
```bash
i-rs-cycling update <ID|DATE> [--distance] [--duration] [--elevation] [--route] [--add-tag] [--remove-tag] [--add-remark]
```

### delete
Delete a cycling record.
```bash
i-rs-cycling delete <ID|DATE>
```

### stats
View cumulative statistics.
```bash
i-rs-cycling stats
```

## Examples

```bash
i-rs-cycling add 2025-06-14 25.5 60 --elevation 300
i-rs-cycling add 2025-06-15 30.2 75 --route "Mountain Trail" --tag mountain
i-rs-cycling list --tag mountain
i-rs-cycling stats
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
