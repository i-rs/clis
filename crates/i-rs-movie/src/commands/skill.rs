use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Movie tracking CLI - manage your personal movie collection.

Key features:
- Track movies with name, year, director, and release date
- Mark movies as watched/unwatched
- Rate movies with 0-10 scale
- Add reviews and remarks
- Filter by tags, watched status
- Statistics overview

Invoke when: managing movie watchlist, tracking watched movies, or recording movie ratings."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <NAME>: Add a movie
- list [--watched|--unwatched|--tag]: List movies
- get <NAME>: Show movie details
- watch <NAME>: Mark movie as watched
- update <NAME>: Update movie info
- delete <NAME>: Delete a movie
- stats: Show statistics
- example: Show examples

Options:
- --year, -y: Release year
- --director, -d: Director name
- --watched, -w: Mark as watched
- --rating, -r: Rating (0-10)
- --review: Review lines (repeatable)
- --release-date: Release date (YYYY-MM-DD)
- --tag, -t: Tags (repeatable)
- --remark: Remark lines (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-movie"
description: "Movie tracking CLI (add/list/get/watch/update/delete/stats). Invoke when user needs to manage their movie collection, track watched movies, or record movie ratings."
---

# i-rs-movie

## Commands

### add
Add a movie to collection.
```bash
i-rs-movie add <NAME> [--year] [--director] [--watched] [--rating] [--review] [--release-date] [--tag] [--remark]
```

### list
List all movies with optional filters.
```bash
i-rs-movie list [--watched|--unwatched] [--tag TAG]
```

### get
Show detailed movie information.
```bash
i-rs-movie get <NAME>
```

### watch
Mark a movie as watched with optional rating.
```bash
i-rs-movie watch <NAME> [--rating] [--review]
```

### update
Update movie information.
```bash
i-rs-movie update <NAME> [--year] [--director] [--rating] [--review] [--release-date] [--tag] [--remark]
```

### delete
Delete a movie from collection.
```bash
i-rs-movie delete <NAME>
```

### stats
Show movie collection statistics.
```bash
i-rs-movie stats
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
