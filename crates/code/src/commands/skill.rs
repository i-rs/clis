use crate::skill_store;

pub fn run_skill_list() {
    let store = skill_store::SkillStore::new();
    let skills = store.list();
    if skills.is_empty() {
        println!("No skills installed.");
        println!();
        println!("Skills directory: {:?}", store.dir());
        println!("Create one with:  i-rs-code skill create <name> --description \"...\"");
        return;
    }
    println!("Skills directory: {:?}", store.dir());
    println!();
    println!("Installed skills ({}):", skills.len());
    for skill in &skills {
        println!("  {}  — {}", skill.name, skill.description);
    }
}

pub fn run_skill_get(name: &str) {
    let store = skill_store::SkillStore::new();
    match store.get(name) {
        Some(skill) => {
            println!("── Skill: {} ──", skill.name);
            println!();
            println!("{}", skill.content);
        }
        None => {
            println!("Skill '{}' not found.", name);
            println!("Use `i-rs-code skill list` to see available skills.");
        }
    }
}

pub fn run_skill_create(name: &str, description: &str) -> anyhow::Result<()> {
    match skill_store::create_skill(name, description) {
        Ok(path) => {
            println!("✓ Skill '{}' created at {:?}", name, path);
            println!("  Edit this file to add your skill instructions.");
            Ok(())
        }
        Err(e) => Err(e),
    }
}
