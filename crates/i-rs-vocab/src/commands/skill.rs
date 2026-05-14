use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Vocabulary learning CLI - manage and review vocabulary words with spaced repetition.

Key features:
- Add vocabulary words with definitions and examples
- Track learning status (new/learning/mastered)
- Quiz mode for active recall practice
- Statistics showing learning progress
- Tag support for organization
- Review count tracking for mastery progression

Invoke when: learning new vocabulary, reviewing words, tracking language learning progress."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <WORD> <DEFINITION>: Add a new vocabulary word
- list [--status] [--tag]: List words with optional filters
- get <WORD>: Show word details
- update <WORD>: Update word properties
- delete <WORD>: Delete a word
- quiz [--count]: Practice vocabulary
- stats: Show learning statistics

Status Values: new, learning, mastered

Options:
- --tag, -t: Tags (repeatable)
- --example, -e: Example sentences (repeatable)
- --status, -s: Initial status
- --definition, -d: Definition
- --remark, -r: Remarks (repeatable)
- --review: Increment review count
- --count: Number of words for quiz"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-vocab"
description: "Vocabulary learning CLI (add/list/get/update/delete/quiz/stats). Invoke when user needs to manage vocabulary words, review vocabulary, or track language learning progress."
---

# i-rs-vocab

## Commands

### add
Add a new vocabulary word.
```bash
i-rs-vocab add <WORD> <DEFINITION> [--tag] [--example] [--status]
```

### list
List vocabulary words.
```bash
i-rs-vocab list [--status STATUS] [--tag TAG]
```

### get
Show word details.
```bash
i-rs-vocab get <WORD>
```

### update
Update a word.
```bash
i-rs-vocab update <WORD> [--definition] [--status] [--tag] [--example] [--review]
```

### delete
Delete a word.
```bash
i-rs-vocab delete <WORD>
```

### quiz
Practice vocabulary.
```bash
i-rs-vocab quiz [--count N]
```

### stats
Show learning statistics.
```bash
i-rs-vocab stats
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
