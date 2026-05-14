use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Wish list tracking CLI - track things you want to buy.

Key features:
- Track items with price and priority
- Mark items as done when purchased
- URL support for products
- Tag organization"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME>: Add wish item
- list [--tag]: List all items
- get <NAME>: Get item details
- update <NAME>: Update item (mark done, etc.)
- delete <NAME>: Delete item"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-want"
description: "Tracks wish list items. Invoke when user wants to track things they want to buy with prices and priorities."
---

# i-rs-want

## Commands

### add
Add a wish item.
```bash
i-rs-want add <NAME> [--url] [--price] [--currency] [--priority] [--tag] [--remark]
```

### list
List all items.
```bash
i-rs-want list [--tag TAG]
```

### get
Get item details.
```bash
i-rs-want get <NAME>
```

### update
Update item.
```bash
i-rs-want update <NAME> [--price] [--priority] [--url] [--tag] [--remark] [--done]
```

### delete
Delete item.
```bash
i-rs-want delete <NAME>
```

## Priorities
- high
- medium
- low"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}
