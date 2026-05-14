pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_DOC: &str = r#"---
name: "i-rs-budget"
description: "Budget management CLI. Track budgets by category with expense tracking."
---

# i-rs-budget

Budget management CLI tool for tracking budgets by category with expense tracking.

## Storage

- Config: `~/.config/i-rs/budget.json`

## Commands

### add
Add a new budget category:
```bash
i-rs-budget add <CATEGORY> <AMOUNT> [--period daily|weekly|monthly|yearly] [--tags TAG...] [--remark REMARK...]
```

### expense
Add an expense to a category:
```bash
i-rs-budget expense <CATEGORY> <AMOUNT> --description <DESC> [--date YYYY-MM-DD] [--tags TAG...]
```

### list
List budgets or expenses:
```bash
i-rs-budget list budgets [--category CATEGORY]
i-rs-budget list expenses [--category CATEGORY]
```

### stats
View budget statistics:
```bash
i-rs-budget stats [--period daily|weekly|monthly|yearly] [--category CATEGORY]
```

### get
Get budget or expense details:
```bash
i-rs-budget get --category <CATEGORY>
i-rs-budget get --expense-id <ID>
```

### update
Update a budget:
```bash
i-rs-budget update <CATEGORY> [--amount AMOUNT] [--period PERIOD] [--tags TAG...] [--remark REMARK...]
```

### delete
Delete a budget or expense:
```bash
i-rs-budget delete --category <CATEGORY>
i-rs-budget delete --expense-id <ID>
```

### example
Show usage examples:
```bash
i-rs-budget example
```

### skill
View AI skill documentation:
```bash
i-rs-budget skill          # Show summary
i-rs-budget skill summary  # Show summary
i-rs-budget skill content # Show full content
i-rs-budget skill raw     # Show raw skill document
```

## JSON Output

All commands support `--json` flag for JSON output:
```bash
i-rs-budget list budgets --json
i-rs-budget stats --json
```

## Examples

```bash
# Create monthly food budget
i-rs-budget add food 500

# Add weekly groceries budget
i-rs-budget add groceries 200 --period weekly

# Track expenses
i-rs-budget expense food 25.50 --description "Lunch"
i-rs-budget expense food 15.00 --description "Coffee"

# View statistics
i-rs-budget stats
i-rs-budget stats --period monthly

# List all
i-rs-budget list budgets
i-rs-budget list expenses
```
"#;

const SKILL_SUMMARY: &str = r#"i-rs-budget - Budget management CLI

Features:
- Create budgets by category with daily/weekly/monthly/yearly periods
- Track expenses per category
- View spending statistics and remaining budget
- Tag and remark support for organization

Storage: ~/.config/i-rs/budget.json

Commands: add, expense, list, stats, get, update, delete, example, skill
"#;

pub fn handle_skill(cmd: Option<SkillCommand>) {
    match cmd {
        Some(SkillCommand::Summary) => {
            println!("{}", SKILL_SUMMARY);
        }
        Some(SkillCommand::Content) | Some(SkillCommand::Raw) | None => {
            println!("{}", SKILL_DOC);
        }
    }
}