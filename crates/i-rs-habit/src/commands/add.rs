use crate::presentation::{print_success, print_header};
use owo_colors::OwoColorize;

pub fn handle_add(name: String, description: String, frequency: String, tags: Vec<String>, remark: Vec<String>) -> anyhow::Result<()> {
    let mut store = crate::storage::load_store()?;
    let habit = crate::service::add_habit(&mut store, name.clone(), description, frequency, tags, remark)?;
    crate::storage::save_store(&store)?;

    print_header("Habit Created");
    println!("{} {}", "Name:".style(owo_colors::Style::new().bold()), habit.name);
    print_success(&format!("Habit '{}' created successfully", name));

    Ok(())
}
