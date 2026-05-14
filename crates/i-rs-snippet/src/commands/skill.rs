use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Code snippet management CLI - store, organize, and quickly retrieve code snippets with language tags and search capabilities.

Key features:
- Store code snippets with language, tags, and descriptions
- Search snippets by name, language, code content, or tags
- Copy snippets to clipboard with one command
- Support multiple programming languages

Invoke when: saving reusable code, finding snippets, or organizing code references."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME>: Add a new snippet with language, code, and metadata
- list [--tag]: List all snippets or filter by tag
- search <QUERY>: Search snippets by name, language, code, or tags
- get <NAME>: Get snippet details with full code
- copy <NAME>: Copy snippet code to clipboard
- update <NAME>: Update snippet info
- delete <NAME>: Delete snippet

Options:
- --language, -l: Programming language (required for add)
- --code, -c: Code lines (repeatable, required for add)
- --description, -d: Description lines (repeatable)
- --tag, -g: Tags (repeatable)
- --remark, -r: Remarks (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-snippet"
description: "Manages code snippets (add/list/get/search/copy/update/delete). Invoke when user needs to save, retrieve, or search code snippets."
---

# i-rs-snippet

## Commands

### add
Add a new code snippet.
```bash
i-rs-snippet add <NAME> --language <LANG> --code <CODE> [--tag] [--description] [--remark]
```

### list
List all snippets.
```bash
i-rs-snippet list [--tag TAG]
```

### search
Search snippets by query.
```bash
i-rs-snippet search <QUERY>
```

### get
Get snippet details.
```bash
i-rs-snippet get <NAME>
```

### copy
Copy snippet to clipboard.
```bash
i-rs-snippet copy <NAME>
```

### update
Update a snippet.
```bash
i-rs-snippet update <NAME> [--language] [--code] [--tag] [--description] [--remark]
```

### delete
Delete a snippet.
```bash
i-rs-snippet delete <NAME>
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
