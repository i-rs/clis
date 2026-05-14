use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Best-by date tracking CLI - track item purchase dates and replacement cycles.

Key features:
- Track item purchase dates
- Set replacement cycles (in days)
- Visual countdown to replacement
- Status alerts: EXPIRED, SOON, OK, NO_CYCLE
- Tag and remark support for organization

Invoke when: tracking item replacement schedules, managing household inventory, or knowing when to replace products."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <PURCHASE_DATE>: Add item with purchase date
- list [--tag]: List all items or filter by tag
- get <NAME>: Get item details
- update <NAME>: Update item info (cycle, tags, etc.)
- delete <NAME>: Delete item

Options:
- --cycle-days, -c: Replacement cycle in days (set via update)
- --tag, -t: Tags (repeatable)
- --remark, -r: Remarks (repeatable)

Status:
- EXPIRED: Past replacement date
- SOON: Within 7 days of replacement
- OK: Replacement not yet needed
- NO_CYCLE: No cycle set (use update --cycle-days)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-bestby"
description: "Manages best-by items (add/list/get/update/delete). Invoke when user needs to track item purchase dates, set replacement cycles, or know when items need replacement."
---

# i-rs-bestby

## Commands

### add
Add a new item.
```bash
i-rs-bestby add <NAME> <PURCHASE_DATE> [--tag] [--remark]
```

### list
List all items.
```bash
i-rs-bestby list [--tag TAG]
```

### get
Get item details.
```bash
i-rs-bestby get <NAME>
```

### update
Update an item's replacement cycle or details.
```bash
i-rs-bestby update <NAME> [--cycle-days DAYS] [--purchase-date DATE] [--tag] [--remark]
```

### delete
Delete an item.
```bash
i-rs-bestby delete <NAME>
```

## Status Values
- **EXPIRED**: Past replacement date (red)
- **SOON**: Within 7 days of replacement (yellow)
- **OK**: Replacement not yet needed (green)
- **NO_CYCLE**: No cycle set"#;

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
