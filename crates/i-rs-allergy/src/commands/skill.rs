use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Allergy tracking CLI - record allergic reactions and symptoms.

Key features:
- Track allergen and severity
- Record symptoms
- Time-based history
- Tag support"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <ALLERGEN> <SEVERITY> [--symptom] [--tag] [--remark]: Record allergy
- list [--tag]: List all records or filter by tag
- get <ID>: Get record details
- delete <ID>: Delete record"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-allergy"
description: "Records allergy reactions. Invoke when user wants to track allergic reactions and symptoms."
---

# i-rs-allergy

## Commands

### add
Record an allergy reaction.
```bash
i-rs-allergy add <ALLERGEN> <SEVERITY> [--symptom] [--tag] [--remark]
```

### list
List all records.
```bash
i-rs-allergy list [--tag TAG]
```

### get
Get record details.
```bash
i-rs-allergy get <ID>
```

### delete
Delete a record.
```bash
i-rs-allergy delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}