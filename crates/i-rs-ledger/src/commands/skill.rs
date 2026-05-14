use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Accounting ledger CLI - track income, expenses, and transfers with summary statistics.

Key features:
- Track income and expenses with categories
- Automatic balance calculation
- Summary view with totals
- Tag and remark support for organization

Invoke when: managing personal finances, tracking income/expenses, or maintaining a simple accounting ledger."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <DATE> <AMOUNT> <CURRENCY> <TYPE> <CATEGORY>: Add entry
- list [--category]: List all entries or filter by category
- get <ID>: Get entry details
- update <ID>: Update entry
- delete <ID>: Delete entry

Options:
- --tag, -t: Tags (repeatable)
- --remark, -r: Remarks (repeatable)
- --amount: Update amount
- --type: income/expense/transfer
- --category: Custom category

Entry Types:
- income: Money received
- expense: Money spent
- transfer: Money moved between accounts"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-ledger"
description: "Manages accounting ledger (add/list/get/update/delete). Invoke when user needs to track income, expenses, transfers, or view financial summary."
---

# i-rs-ledger

## Commands

### add
Add a ledger entry.
```bash
i-rs-ledger add <DATE> <AMOUNT> <CURRENCY> <TYPE> <CATEGORY> [--tag] [--remark]
```

### list
List all entries with summary.
```bash
i-rs-ledger list [--category TYPE]
```

### get
Get entry details.
```bash
i-rs-ledger get <ID>
```

### update
Update an entry.
```bash
i-rs-ledger update <ID> [--date] [--amount] [--type] [--category] [--tag] [--remark]
```

### delete
Delete an entry.
```bash
i-rs-ledger delete <ID>
```

## Entry Types
- **income**: Money received (salary, gifts, etc.)
- **expense**: Money spent (food, rent, etc.)
- **transfer**: Money moved between accounts

## Categories
Custom categories like: salary, food, transport, entertainment, savings, etc."#;

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
