use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Quote collection CLI - store and organize inspirational quotes with authors and sources.

Key features:
- Collect quotes with content, author, and source
- Tag-based organization
- Search by author
- Random quote display
- Personal remarks support

Invoke when: managing inspirational quotes, storing literary excerpts, or organizing wisdom."#;

const SKILL_CONTENT: &str = r#"Commands:
- add: Add a new quote with content, author, source, tags
- list [--tag] [--author]: List all quotes or filter by tag/author
- get <ID>: Get quote details
- delete <ID>: Delete a quote
- random: Display a random quote

Options:
- --content, -c: Quote content
- --author, -a: Quote author
- --source, -s: Quote source (book, speech, etc.)
- --tag, -g: Tags (repeatable)
- --remark, -r: Personal remarks (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-quote"
description: "Manages quotes (add/list/get/delete/random). Invoke when user needs to store, retrieve, or organize inspirational quotes."
---

# i-rs-quote

Quote collection CLI - store and organize inspirational quotes.

## Storage

- Config: `~/.config/i-rs/quotes.json`

## Commands

### add
Add a new quote.
```bash
i-rs-quote add --content "Quote text" [--author "Author"] [--source "Source"] [--tag TAG] [--remark "Note"]
```

### list
List all quotes.
```bash
i-rs-quote list [--tag TAG] [--author AUTHOR]
```

### get
Get quote details.
```bash
i-rs-quote get <ID>
```

### delete
Delete a quote.
```bash
i-rs-quote delete <ID>
```

### random
Display a random quote.
```bash
i-rs-quote random
```

## Examples

```bash
i-rs-quote add --content "The only way to do great work is to love what you do." --author "Steve Jobs"
i-rs-quote add --content "Quote text" --author "Author" --source "Book Name" --tag inspiration --tag life
i-rs-quote list
i-rs-quote list --tag inspiration
i-rs-quote list --author "Steve"
i-rs-quote get <uuid>
i-rs-quote random
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
