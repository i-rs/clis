---
name: "i-rs-goal"
description: "Savings goal tracking CLI tool. Invoke when user wants to track savings goals, set target amounts, record deposits, create milestones, or view financial progress."
---

# i-rs-goal

Savings goal tracking CLI tool for personal finance management.

## Storage

- Config: `~/.config/i-rs/goal.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add
Create a new savings goal.
```bash
i-rs-goal add <name> --target <amount> --deadline <date> [options]
```
- `-t, --target <amount>` - Target amount (required)
- `-d, --deadline <date>` - Deadline YYYY-MM-DD (required)
- `--tags <tags>` - Tags (comma-separated)
- `--remark <remarks>` - Remarks (comma-separated)
- `-m, --milestones <milestones>` - Milestones as name:amount pairs

### list
List all savings goals.
```bash
i-rs-goal list [options]
```
- `--tag <tag>` - Filter by tag

### get
View goal details.
```bash
i-rs-goal get <name>
```

### update
Update goal properties.
```bash
i-rs-goal update <name> [options]
```
- `-t, --target <amount>` - New target amount
- `-d, --deadline <date>` - New deadline
- `--tags <tags>` - New tags

### delete
Delete a goal.
```bash
i-rs-goal delete <name>
```

### deposit
Deposit to a goal.
```bash
i-rs-goal deposit <name> --amount <amount>
```

### milestone
Manage milestones.
```bash
i-rs-goal milestone -g <goal> [options]
```
- `-g, --goal <name>` - Goal name (required)
- `-n, --name <name>` - Milestone name
- `-a, --amount <amount>` - Milestone amount
- `-l, --list` - List milestones only
- `-r, --remove <id>` - Remove milestone by ID

### stats
View statistics.
```bash
i-rs-goal stats [options]
```
- `--tag <tag>` - Filter by tag

### example
Show usage examples.
```bash
i-rs-goal example
```

### data

Manage data (export, import, clear).

```bash
i-rs-goal data export
i-rs-goal data import [FILE]
i-rs-goal data clear
```

### skill

Show skill information.

```bash
i-rs-goal skill [summary|content|raw]
```

## Examples

```bash
# Create a savings goal
i-rs-goal add "Emergency Fund" --target 10000 --deadline 2025-12-31

# Deposit to a goal
i-rs-goal deposit "Emergency Fund" --amount 500

# Add a milestone
i-rs-goal milestone -g "Emergency Fund" -n "First 1000" -a 1000

# View statistics
i-rs-goal stats

# JSON output for scripting
i-rs-goal list --json
```
