use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Inspiration/spark tracking CLI - record moments of inspiration.

Key features:
- Quick capture of ideas and inspirations
- Source tracking (where the idea came from)
- Tag support for categorization
- Time-based history"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <CONTENT>: Record a spark/idea
- list [--tag]: List all sparks or filter by tag
- get <ID>: Get spark details
- delete <ID>: Delete spark"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-spark"
description: "Records moments of inspiration/ideas. Invoke when user wants to capture ideas or sparks of inspiration."
---

# i-rs-spark

## Commands

### add
Record a spark.
```bash
i-rs-spark add <CONTENT> [--source] [--tag] [--remark]
```

### list
List all sparks.
```bash
i-rs-spark list [--tag TAG]
```

### get
Get spark details.
```bash
i-rs-spark get <ID>
```

### delete
Delete a spark.
```bash
i-rs-spark delete <ID>
```"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}
