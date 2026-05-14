---
name: "i-rs-deploy"
description: "Manages deployment records (add/list/get/delete/rollback/stats). Invoke when tracking deployments, managing rollbacks, or viewing deployment statistics."
---

# i-rs-deploy

Deployment record CLI - track deployments, manage rollback, and view statistics.

## Storage

- Config: `~/.config/i-rs/deploy.json`

## Commands

### add
Add a new deployment record.
```bash
i-rs-deploy add <PROJECT> <ENVIRONMENT> <VERSION> [--status STATUS] [--tag] [--remark]
```

### list
List deployment records.
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

## Examples

```bash
# Add deployment
i-rs-deploy add myapp production v1.2.3 --status success --tag frontend

# List deployments
i-rs-deploy list --project myapp

# Rollback
i-rs-deploy rollback myapp production

# View stats
i-rs-deploy stats
```
