use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

pub fn handle_skill(sub: Option<SkillCommand>) {
    let skill_content = include_str!("../../../../skills/i-rs-project/SKILL.md");

    match sub {
        Some(SkillCommand::Summary) => {
            println!("i-rs-project: Project management CLI tool for tracking projects, milestones, and tasks.");
        }
        Some(SkillCommand::Content) | None => {
            println!("{}", skill_content);
        }
        Some(SkillCommand::Raw) => {
            println!("{}", skill_content);
        }
    }
}
