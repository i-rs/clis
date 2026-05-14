use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Bookmark management CLI - store and retrieve website bookmarks with optional credentials.

Key features:
- Store website URLs with optional account credentials
- Credentials stored securely in OS keychain
- Tag and remark support for organization
- Quick retrieval with optional password reveal

Invoke when: managing bookmarks, retrieving saved URLs, or organizing web resources."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <URL>: Add bookmark with optional credentials
- list [--tag]: List all bookmarks or filter by tag
- get <NAME> [--show-password]: Get bookmark details
- update <NAME>: Update bookmark info
- delete <NAME>: Delete bookmark

Options:
- --account, -a: Account/username (optional)
- --password, -p: Password (stored in keychain, optional)
- --tag, -t: Tags (repeatable)
- --remark, -r: Remarks (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-bookmark"
description: "Manages bookmarks (add/list/get/update/delete). Invoke when user needs to store, retrieve, or organize website bookmarks."
---

# i-rs-bookmark

## Commands

### add
Add a new bookmark.
```bash
i-rs-bookmark add <NAME> <URL> [--account] [--password] [--tag] [--remark]
```

### list
List all bookmarks.
```bash
i-rs-bookmark list [--tag TAG]
```

### get
Get bookmark details.
```bash
i-rs-bookmark get <NAME> [--show-password]
```

### update
Update a bookmark.
```bash
i-rs-bookmark update <NAME> [--url] [--account] [--password] [--tag] [--remark]
```

### delete
Delete a bookmark.
```bash
i-rs-bookmark delete <NAME>
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