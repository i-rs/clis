use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Password management CLI - securely store and retrieve website credentials.

Key features:
- Store website URLs with account credentials
- Credentials stored securely in OS keychain
- Tag and remark support for organization
- Quick retrieval with optional password reveal

Invoke when: storing new credentials, retrieving saved passwords, or managing password collections."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <URL>: Add password entry with account credentials
- list [--tag]: List all entries or filter by tag
- get <NAME> [--show-password]: Get entry details
- update <NAME>: Update entry info
- delete <NAME>: Delete entry

Options:
- --account, -a: Account/username
- --password, -p: Password (stored in keychain)
- --tag, -t: Tags (repeatable)
- --remark, -r: Remarks (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-password"
description: "Manages password entries (add/list/get/update/delete). Invoke when user needs to store, retrieve, or manage website credentials."
---

# i-rs-password

## Commands

### add
Add a new password entry.
```bash
i-rs-password add <NAME> <URL> [--account] [--password] [--tag] [--remark]
```

### list
List all password entries.
```bash
i-rs-password list [--tag TAG]
```

### get
Get password entry details.
```bash
i-rs-password get <NAME> [--show-password]
```

### update
Update a password entry.
```bash
i-rs-password update <NAME> [--url] [--account] [--password] [--tag] [--remark]
```

### delete
Delete a password entry.
```bash
i-rs-password delete <NAME>
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