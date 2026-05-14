use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Calorie tracking CLI - estimate and record calorie intake.

Key features:
- Track food items and calorie estimates
- Daily calorie totals
- Tag support for categorization
- Date-based history"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <FOOD_NAME> <CALORIES> [--date] [--tag] [--remark]: Record calorie intake
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-cal"
description: "Records calorie intake estimates. Invoke when user wants to track food calories."
---

# i-rs-cal

## Commands

### add
Record calorie intake.
```bash
i-rs-cal add <FOOD_NAME> <CALORIES> [--date] [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-cal list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-cal get <ID>
```

### delete
Delete a record.
```bash
i-rs-cal delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}