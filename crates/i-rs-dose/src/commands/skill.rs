use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Medication dose tracking CLI - record when you take medicine.

Key features:
- Quick recording of medication intake
- Track dosage and unit
- Time-based history
- Tag support"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <MEDICINE> <DOSAGE> <UNIT>: Record a dose
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-dose"
description: "Records medication intake. Invoke when user wants to track when they take medicine."
---

# i-rs-dose

## Commands

### add
Record a dose.
```bash
i-rs-dose add <MEDICINE> <DOSAGE> <UNIT> [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-dose list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-dose get <ID>
```

### delete
Delete a record.
```bash
i-rs-dose delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}
