use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Birthday reminder CLI - track birthdays with countdown and age calculation.

Key features:
- Track birthdays with MM-DD format
- Calculate age and days until next birthday
- Tag and remark support for organization
- View upcoming birthdays
- Statistics by relationship
- Visual alerts for upcoming birthdays

Invoke when: managing birthday reminders, tracking friends/family birthdays, or scheduling birthday greetings."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <BIRTH_DATE>: Add birthday with MM-DD format
- list [--tag]: List all birthdays or filter by tag
- get <NAME>: Get birthday details with age
- update <NAME>: Update birthday info
- delete <NAME>: Delete birthday
- stats: Show birthday statistics
- upcoming [--days]: Show upcoming birthdays

Options:
- --year, -y: Birth year (optional, for age calculation)
- --relationship, -r: Relationship (e.g., friend, family, colleague)
- --tag, -g: Tags (repeatable)
- --remark: Remark lines (repeatable)
- --days: Number of days to look ahead (default: 30)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-birthday"
description: "Manages birthday reminders (add/list/get/update/delete/stats/upcoming). Invoke when user needs to track birthdays, calculate ages, or view upcoming birthdays."
---

# i-rs-birthday

## Commands

### add
Add a new birthday.
```bash
i-rs-birthday add <NAME> <BIRTH_DATE> [--year] [--relationship] [--tag] [--remark]
```

### list
List all birthdays.
```bash
i-rs-birthday list [--tag TAG]
```

### get
Get birthday details.
```bash
i-rs-birthday get <NAME>
```

### update
Update a birthday.
```bash
i-rs-birthday update <NAME> [--birth-date] [--year] [--relationship] [--tag] [--remark]
```

### delete
Delete a birthday.
```bash
i-rs-birthday delete <NAME>
```

### stats
Show birthday statistics.
```bash
i-rs-birthday stats
```

### upcoming
Show upcoming birthdays.
```bash
i-rs-birthday upcoming [--days DAYS]
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
