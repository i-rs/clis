use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Subscription tracking CLI - track recurring subscriptions with renewal reminders.

Key features:
- Track subscriptions with billing cycles
- Automatic next billing date calculation
- Status alerts: EXPIRED, DUE_SOON, OK
- URL and tag support"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <AMOUNT> <CURRENCY> <CYCLE> <START_DATE>: Add subscription
- list [--tag]: List all subscriptions
- get <NAME>: Get subscription details
- update <NAME>: Update subscription
- delete <NAME>: Delete subscription"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-sub"
description: "Manages subscriptions with renewal reminders. Invoke when user needs to track recurring subscriptions and know when they need to renew."
---

# i-rs-sub

## Commands

### add
Add a subscription.
```bash
i-rs-sub add <NAME> <AMOUNT> <CURRENCY> <CYCLE> <START_DATE> [--url] [--tag] [--remark]
```

### list
List all subscriptions.
```bash
i-rs-sub list [--tag TAG]
```

### get
Get subscription details.
```bash
i-rs-sub get <NAME>
```

### update
Update subscription.
```bash
i-rs-sub update <NAME> [--amount] [--cycle] [--next-date] [--url] [--tag] [--remark]
```

### delete
Delete subscription.
```bash
i-rs-sub delete <NAME>
```

## Billing Cycles
- daily
- weekly
- monthly
- quarterly
- yearly"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}
