use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Key storage CLI - securely store API keys, tokens, and sensitive credentials in OS keychain.

Key features:
- Store sensitive keys securely in OS keychain
- Track key metadata (type, tags, remarks)
- Never expose keys in config files
- Tag and remark support for organization

Invoke when: managing API keys, tokens, passwords, or any sensitive credentials that should not be stored in plain text."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <VALUE> <TYPE>: Add key with secure storage
- list [--tag]: List all keys or filter by tag
- get <NAME>: Get key details
- update <NAME>: Update key metadata
- delete <NAME>: Delete key

Options:
- --tag, -t: Tags (repeatable)
- --remark, -r: Remarks (repeatable)
- --type: Key type (api_key, token, password, etc.)
- --show-value: Reveal stored value

Security:
- Values stored in OS keychain (Keychain/macOS, Credential Manager/Windows)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-keys"
description: "Manages secure key storage (add/list/get/update/delete). Invoke when user needs to store API keys, tokens, passwords, or sensitive credentials securely in OS keychain."
---

# i-rs-keys

## Commands

### add
Add a key with secure storage.
```bash
i-rs-keys add <NAME> <VALUE> <TYPE> [--tag] [--remark]
```

### list
List all keys.
```bash
i-rs-keys list [--tag TAG]
```

### get
Get key details.
```bash
i-rs-keys get <NAME> [--show-value]
```

### update
Update key metadata or value.
```bash
i-rs-keys update <NAME> [--key-value VALUE] [--type TYPE] [--tag] [--remark]
```

### delete
Delete a key.
```bash
i-rs-keys delete <NAME>
```

## Key Types
- api_key
- token
- password
- certificate
- ssh_key
- etc.

## Security
Values are stored in OS keychain:
- macOS: Keychain
- Windows: Credential Manager
- Linux: libsecret

Keys are NEVER stored in config files or JSON."#;

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
