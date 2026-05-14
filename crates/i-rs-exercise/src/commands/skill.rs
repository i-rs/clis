use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Exercise tracking CLI - track your workouts and fitness activities with detailed statistics.

Key features:
- Record exercises with type, duration, and calories
- Categorize with tags for easy filtering
- Add notes and remarks for context
- View comprehensive statistics by exercise type
- Track total duration and calories burned

Invoke when: logging workouts, tracking fitness progress, or viewing exercise statistics."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <TYPE> <DURATION> [CALORIES]: Add exercise record
- list [--tag TAG] [--type TYPE]: List exercises with optional filters
- get <NAME>: Get exercise details
- update <NAME>: Update exercise record
- delete <NAME>: Delete exercise record
- stats: Show exercise statistics
- example: Show usage examples

Options:
- --tag, -t: Filter by tag (repeatable)
- --type, -y: Filter by exercise type
- --notes, -n: Add notes (repeatable)
- --remark, -r: Add remarks (repeatable)
- --calories, -c: Calories burned
- --duration, -d: Duration in minutes"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-exercise"
description: "Tracks exercise and fitness activities (add/list/update/delete/stats). Invoke when user needs to record workouts, track fitness progress, or display exercise statistics."
---

# i-rs-exercise

## Commands

### add
Add an exercise record.
```bash
i-rs-exercise add <NAME> <TYPE> <DURATION> [--calories] [--tag] [--notes] [--remark]
```

### list
List exercise records.
```bash
i-rs-exercise list [--tag TAG] [--type TYPE]
```

### get
Get exercise details.
```bash
i-rs-exercise get <NAME>
```

### update
Update an exercise record.
```bash
i-rs-exercise update <NAME> [--type] [--duration] [--calories] [--notes] [--tag] [--remark]
```

### delete
Delete an exercise record.
```bash
i-rs-exercise delete <NAME>
```

### stats
Show exercise statistics.
```bash
i-rs-exercise stats
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
