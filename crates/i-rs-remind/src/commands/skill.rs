use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Reminder management CLI - track events and deadlines with countdown.

Key features:
- Track event dates with countdown
- Mark events as done
- Tag and remark support for organization
- Visual alerts for upcoming/past events
- Multi-line content support

Invoke when: managing reminders, tracking deadlines, or scheduling events."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <EVENT_DATE>: Add reminder with event date
- list [--tag]: List all reminders or filter by tag
- get <NAME>: Get reminder details
- done <NAME>: Mark reminder as done
- update <NAME>: Update reminder info
- delete <NAME>: Delete reminder

Options:
- --title, -t: Event title (optional)
- --tag, -g: Tags (repeatable)
- --content, -c: Content lines (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-remind"
description: "Manages reminders (add/list/get/update/delete/done). Invoke when user needs to track events, deadlines, or schedule reminders."
---

# i-rs-remind

## Commands

### add
Add a new reminder.
```bash
i-rs-remind add <NAME> <EVENT_DATE> [--title] [--tag] [--content]
```

### list
List all reminders.
```bash
i-rs-remind list [--tag TAG]
```

### get
Get reminder details.
```bash
i-rs-remind get <NAME>
```

### done
Mark reminder as done.
```bash
i-rs-remind done <NAME>
```

### update
Update a reminder.
```bash
i-rs-remind update <NAME> [--event-date] [--title] [--tag] [--content]
```

### delete
Delete a reminder.
```bash
i-rs-remind delete <NAME>
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