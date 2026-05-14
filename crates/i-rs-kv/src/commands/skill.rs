use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Key-Value storage CLI - store and retrieve simple key-value data.

Key features:
- Store arbitrary key-value pairs
- Tag and remark support for organization
- Quick retrieval of stored values
- JSON output support

Invoke when: storing simple configuration data, API keys, or quick lookup values."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <KEY> <VALUE>: Add key-value pair
- list [--tag]: List all entries or filter by tag
- get <KEY>: Get value by key
- update <KEY>: Update value or tags
- delete <KEY>: Delete entry

Options:
- --tag, -t: Tags (repeatable)
- --remark, -r: Remarks (repeatable)
- --value: New value for update"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-kv"
description: "Manages key-value storage (add/list/get/update/delete). Invoke when user needs to store or retrieve simple key-value data like API keys or configuration."
---

# i-rs-kv

## Commands

### add
Add a new key-value entry.
```bash
i-rs-kv add <KEY> <VALUE> [--tag] [--remark]
```

### list
List all entries.
```bash
i-rs-kv list [--tag TAG]
```

### get
Get value by key.
```bash
i-rs-kv get <KEY>
```

### update
Update an entry.
```bash
i-rs-kv update <KEY> [--value VALUE] [--tag] [--remark]
```

### delete
Delete an entry.
```bash
i-rs-kv delete <KEY>
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
