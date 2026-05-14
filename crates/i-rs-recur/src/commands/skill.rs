use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Recurring expense tracking CLI - track fixed expenses with automatic next payment calculation.

Key features:
- Track recurring payments (daily/weekly/monthly/quarterly/yearly)
- Automatic next payment date calculation
- Tag and remark support
- Visual alerts for upcoming/overdue payments

Invoke when: managing subscriptions, tracking recurring bills, or planning monthly expenses."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <AMOUNT> <CURRENCY> <FREQUENCY> <START_DATE>: Add expense
- list [--tag]: List all expenses or filter by tag
- get <NAME>: Get expense details
- update <NAME>: Update expense info
- delete <NAME>: Delete expense

Options:
- --tag, -t: Tags (repeatable)
- --remark, -r: Remarks (repeatable)
- --amount: Update amount
- --frequency: daily/weekly/monthly/quarterly/yearly
- --start-date: Update start date"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-recur"
description: "Manages recurring expenses (add/list/get/update/delete). Invoke when user needs to track subscriptions, recurring bills, or fixed payments with automatic due date calculation."
---

# i-rs-recur

## Commands

### add
Add a recurring expense.
```bash
i-rs-recur add <NAME> <AMOUNT> <CURRENCY> <FREQUENCY> <START_DATE> [--tag] [--remark]
```

### list
List all expenses.
```bash
i-rs-recur list [--tag TAG]
```

### get
Get expense details.
```bash
i-rs-recur get <NAME>
```

### update
Update an expense.
```bash
i-rs-recur update <NAME> [--amount AMOUNT] [--frequency FREQ] [--start-date DATE] [--tag] [--remark]
```

### delete
Delete an expense.
```bash
i-rs-recur delete <NAME>
```

## Frequencies
- daily
- weekly
- monthly
- quarterly
- yearly"#;

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
