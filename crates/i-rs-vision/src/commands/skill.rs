use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Vision tracking CLI - track eye prescription (sphere, cylinder, axis) over time.

Key features:
- Record vision measurements (sphere, cylinder, axis)
- View prescription history
- Track vision changes over time
- Calculate vision statistics
- Optional tags and remarks

Invoke when: tracking eye prescription changes, monitoring myopia progression, or maintaining vision history."#;

const SKILL_CONTENT: &str = r#"Commands:
- add <DATE>: Add vision record with sphere/cylinder/axis values
- list [--days]: List records with optional time filter
- get <DATE>: Get specific record details
- delete <DATE>: Delete a record
- stats: Show vision statistics

Options:
- -ls, --left-sphere: Left eye sphere (D)
- -rs, --right-sphere: Right eye sphere (D)
- -lc, --left-cylinder: Left eye cylinder (D)
- -rc, --right-cylinder: Right eye cylinder (D)
- -la, --left-axis: Left eye axis (degrees)
- -ra, --right-axis: Right eye axis (degrees)
- -t, --tag: Tags (repeatable)
- -r, --remark: Remarks (repeatable)"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-vision"
description: "Tracks vision prescription (add/list/get/delete). Invoke when user needs to record eye measurements, view vision history, or display statistics."
---

# i-rs-vision

## Commands

### add
Add a vision record.
```bash
i-rs-vision add <DATE> [--left-sphere] [--right-sphere] [--left-cylinder] [--right-cylinder] [--left-axis] [--right-axis] [--tag] [--remark]
```

### list
List vision records.
```bash
i-rs-vision list [--days N]
```

### get
Get a specific record.
```bash
i-rs-vision get <DATE>
```

### delete
Delete a vision record.
```bash
i-rs-vision delete <DATE>
```

### stats
Show vision statistics.
```bash
i-rs-vision stats
```

## Vision Parameters

- **Sphere**: Refractive error (negative = myopia, positive = hyperopia)
- **Cylinder**: Astigmatism correction
- **Axis**: Orientation of astigmatism (0-180 degrees)"#;

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
