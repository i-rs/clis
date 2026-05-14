use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Domain management CLI - track domain expiry dates and registrar information.

Key features:
- Track domain expiry dates with countdown
- Store registrar information
- Credentials stored securely in OS keychain
- Tag and remark support for organization
- Visual alerts for expiring/expired domains

Invoke when: managing domain portfolio, tracking renewals, or retrieving domain details."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <EXPIRY_DATE>: Add domain with expiry date
- list [--tag]: List all domains or filter by tag
- get <NAME> [--show-password]: Get domain details
- update <NAME>: Update domain info
- delete <NAME>: Delete domain

Options:
- --registrar, -r: Registrar name
- --password, -p: Registrar password (keychain)
- --tag, -t: Tags (repeatable)
- --remark, -m: Remarks (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-domain"
description: "Manages domains (add/list/get/update/delete). Invoke when user needs to track domain expiry dates, manage registrar credentials, or organize domain portfolio."
---

# i-rs-domain

## Commands

### add
Add a new domain.
```bash
i-rs-domain add <NAME> <EXPIRY_DATE> [--registrar] [--password] [--tag] [--remark]
```

### list
List all domains.
```bash
i-rs-domain list [--tag TAG]
```

### get
Get domain details.
```bash
i-rs-domain get <NAME> [--show-password]
```

### update
Update a domain.
```bash
i-rs-domain update <NAME> [--expiry-date] [--registrar] [--password] [--tag] [--remark]
```

### delete
Delete a domain.
```bash
i-rs-domain delete <NAME>
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