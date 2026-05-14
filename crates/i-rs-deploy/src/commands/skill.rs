use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Deployment record CLI - track deployments, manage rollback, and view statistics.

Key features:
- Track deployments across projects and environments
- Record deployment status (success/failed/rollback)
- Support for rollback tracking
- Statistics and timeline views
- Tag and remark support

Invoke when: managing deployment history, tracking rollbacks, or viewing deployment statistics."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <PROJECT> <ENV> <VERSION>: Add deployment record
- list [--project] [--env]: List deployments
- get <ID>: Get deployment details
- delete <ID>: Delete deployment record
- rollback <PROJECT> <ENV>: Rollback to previous version
- stats [--project] [--env]: Show statistics
- example: Show usage examples

Status options: success, failed, rolling_back, rolled_back

Options:
- --tag, -t: Tags (repeatable)
- --remark, -r: Remarks (repeatable)
- --rollback-to: Specific deployment ID to rollback to"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-deploy"
description: "Manages deployment records (add/list/get/delete/rollback/stats). Invoke when tracking deployments, managing rollbacks, or viewing deployment statistics."
---

# i-rs-deploy

## Commands

### add
Add a new deployment record.
```bash
i-rs-deploy add <PROJECT> <ENVIRONMENT> <VERSION> [--status STATUS] [--tag] [--remark]
```

### list
List all deployment records.
```bash
i-rs-deploy list [--project PROJECT] [--env ENVIRONMENT] [--tag TAG]
```

### get
Get deployment details by ID.
```bash
i-rs-deploy get <ID>
```

### delete
Delete a deployment record.
```bash
i-rs-deploy delete <ID>
```

### rollback
Rollback to previous deployment.
```bash
i-rs-deploy rollback <PROJECT> <ENVIRONMENT> [--rollback-to ID]
```

### stats
Show deployment statistics.
```bash
i-rs-deploy stats [--project PROJECT] [--env ENVIRONMENT]
```

### example
Show usage examples.
```bash
i-rs-deploy example
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
