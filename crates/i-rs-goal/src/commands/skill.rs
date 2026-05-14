use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
pub struct SkillArgs {
    #[arg(long, help = "Show skill summary")]
    pub summary: bool,
    
    #[arg(long, help = "Show skill content")]
    pub content: bool,
}

pub fn skill(args: SkillArgs) -> anyhow::Result<()> {
    let skill_path = format!("{}/skills/i-rs-goal/SKILL.md", 
        std::env::current_exe()?.parent().unwrap().parent().unwrap().display());
    
    if args.summary {
        println!("i-rs-goal: Savings goal tracker CLI tool");
        println!("Commands: add, list, get, delete, update, deposit, milestone, stats, example, skill");
        return Ok(());
    }
    
    let content = if std::path::Path::new(&skill_path).exists() {
        fs::read_to_string(&skill_path)?
    } else {
        get_inline_skill_doc().to_string()
    };
    
    println!("{}", content);
    
    Ok(())
}

fn get_inline_skill_doc() -> &'static str {
    r#"# i-rs-goal Skill

Savings goal tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/goal.json`

## Commands

### add
Create a new savings goal.
```bash
i-rs-goal add "Emergency Fund" --target 10000 --deadline 2025-12-31 --tags emergency,finance
```

### list
List all savings goals.
```bash
i-rs-goal list
i-rs-goal list --tag emergency
```

### get
View goal details.
```bash
i-rs-goal get "Emergency Fund"
```

### deposit
Deposit to a goal.
```bash
i-rs-goal deposit "Emergency Fund" --amount 500
```

### milestone
Manage milestones.
```bash
i-rs-goal milestone -g "Emergency Fund" -n "First 1000" -a 1000
i-rs-goal milestone -g "Emergency Fund" --list
i-rs-goal milestone -g "Emergency Fund" -r <milestone-id>
```

### update
Update goal properties.
```bash
i-rs-goal update "Emergency Fund" --target 15000
```

### delete
Delete a goal.
```bash
i-rs-goal delete "Emergency Fund"
```

### stats
View statistics.
```bash
i-rs-goal stats
i-rs-goal stats --tag emergency
```

### example
Show usage examples.
```bash
i-rs-goal example
```

## Features

- Track multiple savings goals
- Set target amounts and deadlines
- Record deposits
- Create milestones
- Tag support
- Progress tracking
- JSON output for scripting
"#
}
