use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Meal tracking CLI - record daily meals (breakfast, lunch, dinner).

Key features:
- Track meals by type (breakfast, lunch, dinner, snack)
- Optional calorie tracking
- View meals by date
- Tag support"#;

const SKILL_CONTENT: &str = r#"Commands:
- add <TYPE> <FOOD> <DATE>: Add a meal
- list [--date]: List today's meals or by specific date
- get <ID>: Get meal details
- delete <ID>: Delete meal"#;

const SKILL_RAW: &str = r#"---
name: "i-rs-meal"
description: "Records daily meals (breakfast, lunch, dinner, snacks). Invoke when user wants to track what they eat."
---

# i-rs-meal

## Commands

### add
Add a meal.
```bash
i-rs-meal add <TYPE> <FOOD_ITEMS> <DATE> [--calories] [--tag] [--remark]
```

### list
List meals.
```bash
i-rs-meal list [--date YYYY-MM-DD]
```

### get
Get meal details.
```bash
i-rs-meal get <ID> [--date YYYY-MM-DD]
```

### delete
Delete a meal.
```bash
i-rs-meal delete <ID>
```

## Meal Types
- breakfast
- lunch
- dinner
- snack"#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}
