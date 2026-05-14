use anyhow::Result;
use clap::Parser;

const SKILL_CONTENT: &str = r#"---
name: "i-rs-debt"
description: "Debt management CLI tool. Track credit card debts, loans, borrowed money with payment history and overdue reminders."
---

# i-rs-debt

Debt management CLI tool for tracking credit card debts, loans, and borrowed money.

## Storage

- Config: `~/.config/i-rs/debt.json`

## Commands

### add
Create a new debt entry.
```bash
i-rs-debt add <name> --debt-type <type> --amount <amount> [options]
```
Options:
- `--debt-type`: credit_card, loan, borrowed
- `--amount`: Total debt amount
- `--interest-rate`: Interest rate percentage (optional)
- `--due-date`: Due date YYYY-MM-DD (optional)
- `--tags`: Comma-separated tags (optional)
- `--remark`: Remarks (optional, multiple)

### list
List all debts with optional filters.
```bash
i-rs-debt list [options]
```
Options:
- `--tag`: Filter by tag
- `--overdue`: Show overdue debts only
- `--paid-off`: Show paid off debts only

### get
Show detailed debt information.
```bash
i-rs-debt get <name> [options]
```
Options:
- `--payments`: Show payment history

### pay
Record a payment for a debt.
```bash
i-rs-debt pay <name> --amount <amount> [options]
```
Options:
- `--amount`: Payment amount
- `--note`: Payment note (optional)

### update
Update debt information.
```bash
i-rs-debt update <name> [options]
```
Options:
- `--rename`: New name
- `--debt-type`: New type
- `--amount`: New total amount
- `--interest-rate`: New interest rate
- `--due-date`: New due date
- `--add-tags`: Add tags (comma-separated)
- `--remove-tags`: Remove tags (comma-separated)
- `--add-remark`: Add remarks

### delete
Delete a debt entry.
```bash
i-rs-debt delete <name> [options]
```
Options:
- `--force`: Skip confirmation

### stats
Show debt statistics.
```bash
i-rs-debt stats [options]
```
Options:
- `--by-type`: Show statistics by debt type

### example
Show usage examples.
```bash
i-rs-debt example
```

### skill
Show skill documentation.
```bash
i-rs-debt skill
i-rs-debt skill summary
```

## JSON Output

All commands support `--json` flag for JSON output:
```bash
i-rs-debt list --json
i-rs-debt get <name> --json
i-rs-debt stats --json
```
"#;

const SKILL_SUMMARY: &str = r#"i-rs-debt: Debt management CLI for tracking credit card debts, loans, and borrowed money with payment history and overdue reminders.

Commands: add, list, get, pay, update, delete, stats, example, skill
"#;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, help = "Show summary only")]
    pub summary: bool,
}

pub fn run(args: &Args) -> Result<()> {
    if args.summary {
        println!("{}", SKILL_SUMMARY);
    } else {
        println!("{}", SKILL_CONTENT);
    }
    Ok(())
}
