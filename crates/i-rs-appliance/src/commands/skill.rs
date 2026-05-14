use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Home appliance lifecycle management CLI - track appliances with lifespan, maintenance records, and replacement reminders.

Key features:
- Record appliances with brand, model, purchase date
- Track expected lifespan in years
- Add maintenance records
- View expiry status and days remaining
- Filter by tags

Invoke when: managing home appliances, tracking replacement schedules, or recording maintenance history."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <BRAND> <MODEL> <PURCHASE_DATE> <LIFESPAN>: Add appliance
- list [--tag]: List all appliances, optionally filtered by tag
- get <NAME>: Show appliance details with maintenance history
- update <NAME>: Update appliance info or add maintenance
- delete <NAME>: Remove appliance
- stats: Show statistics overview

Options:
- --tag, -t: Filter by tag
- --lifespan, -l: Expected lifespan in years
- --remark, -r: Remarks (repeatable)
- --add-maintenance: Add maintenance record
- --maintenance-date: Maintenance record date"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-appliance"
description: "Manages home appliance lifecycle (add/list/get/update/delete). Invoke when tracking appliance lifespans, recording maintenance, or setting replacement reminders."
---

# i-rs-appliance

## Commands

### add
Add a new appliance.
```bash
i-rs-appliance add <NAME> <BRAND> <MODEL> <PURCHASE_DATE> <LIFESPAN_YEARS> [--tag] [--remark]
```

### list
List all appliances.
```bash
i-rs-appliance list [--tag TAG]
```

### get
Show appliance details.
```bash
i-rs-appliance get <NAME>
```

### update
Update appliance or add maintenance record.
```bash
i-rs-appliance update <NAME> [--brand] [--model] [--lifespan] [--tag] [--remark] [--add-maintenance] [--maintenance-date]
```

### delete
Delete appliance.
```bash
i-rs-appliance delete <NAME>
```

### stats
Show statistics.
```bash
i-rs-appliance stats
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
