use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Article read-later CLI - save and organize articles for later reading.

Key features:
- Save articles with title, URL, and source
- Track reading status (unread/reading/read)
- Add notes and tags for organization
- View reading statistics

Invoke when: saving articles to read later, tracking reading progress, or organizing reading lists."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME> <URL> <TITLE>: Add new article
- list [--tag] [--status]: List articles (filter by tag or status)
- get <NAME>: Get article details
- read <NAME>: Mark article as read
- update <NAME>: Update article info
- delete <NAME>: Delete article
- stats: Show reading statistics

Options:
- --title, -t: Article title
- --source, -s: Article source (e.g., blog name, website)
- --tag, -g: Tags (repeatable)
- --remark, -r: Remarks (repeatable)
- --notes, -n: Reading notes (repeatable)
- --status: Reading status (unread/reading/read)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-article"
description: "Article read-later management (add/list/get/read/update/delete). Invoke when user wants to save articles for later reading."
---

# i-rs-article

## Storage

- Config: `~/.config/i-rs/articles.json`

## Commands

### add
Add a new article.
```bash
i-rs-article add <NAME> <URL> <TITLE> [--source] [--tag] [--remark]
```

### list
List all articles or filter by tag/status.
```bash
i-rs-article list [--tag TAG] [--status STATUS]
```

### get
Get article details.
```bash
i-rs-article get <NAME>
```

### read
Mark article as read.
```bash
i-rs-article read <NAME>
```

### update
Update article information.
```bash
i-rs-article update <NAME> [--title] [--url] [--source] [--status] [--tag] [--remark] [--notes]
```

### delete
Delete an article.
```bash
i-rs-article delete <NAME>
```

### stats
Show reading statistics.
```bash
i-rs-article stats
```

## Examples

```bash
i-rs-article add rust-blog https://rust-lang.org/blog \"Rust Blog\" --source rust-lang.org --tag programming
i-rs-article list --status unread
i-rs-article read rust-blog
i-rs-article stats
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
