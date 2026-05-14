use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Gift management CLI - track gifts given and received with statistics.

Key features:
- Record gifts with type (sent/received), recipient, occasion, and value
- Tag and remark support for organization
- Statistics on gift exchanges and spending
- Filter by type or tag

Invoke when: managing gift records, tracking gift exchanges, or viewing gift statistics."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <TYPE> <RECIPIENT> <OCCASION> <VALUE> <DATE>: Add a gift
- list [--type] [--tag]: List all gifts or filter by type/tag
- get <NAME>: Get gift details
- delete <NAME>: Delete a gift
- stats: Show gift statistics
- example: Show usage examples

Options:
- TYPE: 'sent' or 'received'
- OCCASION: birthday, christmas, anniversary, etc.
- VALUE: monetary value (floating point)
- DATE: date in YYYY-MM-DD format
- --tag, -t: Tags (repeatable)
- --remark, -r: Remarks (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-gift"
description: "Manages gift records (add/list/get/delete/stats). Invoke when user needs to track gifts given or received, or view gift exchange statistics."
---

# i-rs-gift

## Commands

### add
Add a new gift record.
```bash
i-rs-gift add <NAME> <TYPE> <RECIPIENT> <OCCASION> <VALUE> <DATE> [--tag] [--remark]
```

### list
List all gifts or filter by type/tag.
```bash
i-rs-gift list [--type TYPE] [--tag TAG]
```

### get
Get gift details.
```bash
i-rs-gift get <NAME>
```

### delete
Delete a gift.
```bash
i-rs-gift delete <NAME>
```

### stats
Show gift statistics.
```bash
i-rs-gift stats
```

### example
Show usage examples.
```bash
i-rs-gift example
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
