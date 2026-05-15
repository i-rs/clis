use crate::presentation::{print_success, print_header};
use owo_colors::OwoColorize;

pub fn handle_add(name: String, description: String, frequency: String, tags: Vec<String>, remark: Vec<String>) -> anyhow::Result<()> {
    let habit = crate::service::add_habit(name.clone(), description, frequency, tags, remark)?;

    print_header("Habit Created");
    println!("{} {}", "Name:".style(owo_colors::Style::new().bold()), habit.name);
    print_success(&format!("Habit '{}' created successfully", name));

    Ok(())
}
