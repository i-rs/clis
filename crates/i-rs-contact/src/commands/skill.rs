use owo_colors::OwoColorize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

pub fn handle_skill(subcommand: Option<SkillCommand>) {
    let skill_path = match std::env::var("SKILLS_DIR") {
        Ok(dir) => PathBuf::from(dir).join("i-rs-contact").join("SKILL.md"),
        Err(_) => {
            let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
            let root_dir = exe_path.parent().unwrap().parent().unwrap().parent().unwrap();
            root_dir.join("skills").join("i-rs-contact").join("SKILL.md")
        }
    };

    let content = match fs::read_to_string(&skill_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("{}", "Skill documentation not found.".red());
            return;
        }
    };

    match subcommand {
        Some(SkillCommand::Summary) => {
            let lines: Vec<&str> = content.lines().take(20).collect();
            println!("{}", lines.join("\n"));
        }
        Some(SkillCommand::Content) => {
            let start_idx = content.find("# i-rs-contact").unwrap_or(0);
            let end_idx = content.find("## Examples").unwrap_or(content.len());
            println!("{}", &content[start_idx..end_idx]);
        }
        Some(SkillCommand::Raw) => {
            println!("{}", content);
        }
        None => {
            let start_idx = content.find("# i-rs-contact").unwrap_or(0);
            println!("{}", &content[start_idx..]);
        }
    }
}
