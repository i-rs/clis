use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct SkillArgs {
    #[arg(help = "Show summary only", default_value = "false")]
    pub summary: bool,

    #[arg(help = "Show content only", default_value = "false")]
    pub content: bool,
}

pub fn skill(args: SkillArgs) -> Result<()> {
    let skill_doc = r#"---
name: "i-rs-read"
description: "Reading progress tracking CLI tool. Track books, pages read, and reading statistics."
---

# i-rs-read

Reading progress tracking CLI tool for managing your book collection.

## Storage

- Config: `~/.config/i-rs/read.json`

## Commands

### add
Add a new book to your reading list.
```bash
i-rs-read add <name> <author> <total_pages> [--tags <tags>] [--remark <remark>]
```

### list
List all books with optional filters.
```bash
i-rs-read list [--tag <tag>] [--status <status>]
```

### get
Show detailed information about a book.
```bash
i-rs-read get <name>
```

### update
Update book information (progress, status, rating, etc.).
```bash
i-rs-read update <name> [--current-page <n>] [--status <status>] [--rating <n>] [--review <text>] [--tags <tags>] [--add-remark <text>] [--remove-remark <n>]
```

### delete
Remove a book from your reading list.
```bash
i-rs-read delete <name>
```

### stats
Display reading statistics.
```bash
i-rs-read stats [--tag <tag>]
```

### example
Show usage examples.
```bash
i-rs-read example
```

## Status Options

- `reading` - Currently reading
- `completed` - Finished reading
- `paused` - Paused reading
- `dropped` - Dropped/abandoned
- `to_read` - Plan to read

## Examples

```bash
# Add a book
i-rs-read add "The Rust Programming Language" "Steve Klabnik" 500

# Track progress
i-rs-read update "The Rust Programming Language" --current-page 250 --status reading

# Mark as completed with rating
i-rs-read update "The Rust Programming Language" --status completed --rating 5

# View statistics
i-rs-read stats
```"#;

    if args.summary {
        println!("Reading progress tracking CLI tool. Track books, pages read, and reading statistics.");
    } else if args.content {
        println!("{}", skill_doc);
    } else {
        println!("{}", skill_doc);
    }

    Ok(())
}
