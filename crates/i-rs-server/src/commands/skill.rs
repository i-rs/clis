use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

pub fn handle_skill(which: Option<SkillCommand>) {
    const SKILL_SUMMARY: &str = r#"Server management CLI - manage server inventory and SSH connection details.

Key features:
- Store server host, port, user credentials
- Credentials stored securely in OS keychain
- Tag and remark support for organization
- SSH command suggestions

Invoke when: managing server inventory, retrieving connection details, or generating SSH commands."#;

    const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <HOST> [PORT]: Add server with credentials
- list [--tag]: List all servers or filter by tag
- get <NAME> [--show-password]: Get server details
- update <NAME>: Update server info
- delete <NAME>: Delete server
- suggest <NAME>: Get SSH command suggestions

Options:
- -u, --user: SSH username
- -p, --password: Password (keychain)
- -t, --tag: Tags (repeatable)
- -r, --remark: Remarks (repeatable)"#;

    const SKILL_RAW: &str = r#"---
name: "i-rs-server"
description: "Manages server configurations (add/list/get/update/delete/suggest). Invoke when user needs to manage server inventory or retrieve server connection details."
---

# i-rs-server

## Commands

### add
Add a new server.
```bash
i-rs-server add <NAME> <HOST> [PORT]
```

### list
List servers.
```bash
i-rs-server list [--tag TAG]
```

### get
Get server details.
```bash
i-rs-server get <NAME> [--show-password]
```

### update
Update server.
```bash
i-rs-server update <NAME> [--host HOST] [--port PORT] [--user USER] [--password PASSWORD] [--tag TAG] [--remark REMARK]
```

### delete
Delete server.
```bash
i-rs-server delete <NAME>
```

### suggest
Get SSH command suggestions.
```bash
i-rs-server suggest <NAME> [--command CMD]
```

## Security
Passwords stored in OS keychain, never in config file.

## Storage
- Config: ~/.config/i-rs/servers.json
- Passwords: OS Keychain"#;

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
