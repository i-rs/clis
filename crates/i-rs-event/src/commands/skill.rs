use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct SkillArgs {
    #[arg(help = "Show 'summary' or 'content' (default: content)")]
    pub which: Option<String>,
}

pub fn run(args: &SkillArgs) -> Result<()> {
    let skill_content = include_str!("../../../../skills/i-rs-event/SKILL.md");

    match args.which.as_deref() {
        Some("summary") => {
            println!("i-rs-event: CLI tool for social event management. Supports add/list/get/delete events with types (meeting/gathering/course/other), tags, and yearly statistics.");
        }
        Some("content") | None => {
            println!("{}", skill_content);
        }
        _ => {
            println!("Usage: skill [summary|content]");
        }
    }

    Ok(())
}
