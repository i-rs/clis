use crate::presentation::{print_error, print_success, print_header};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_add(name: String, description: String, frequency: String, tags: Vec<String>, remark: Vec<String>) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    if store.get_entry(&name).is_some() {
        print_error(&format!("Habit '{}' already exists", name));
        anyhow::bail!("Habit '{}' already exists", name);
    }

    let _habit = storage::add_habit(&mut store, name.clone(), description, frequency, tags, remark)?;
    storage::save_store(&store)?;

    print_header("Habit Created");
    println!("{} {}", "Name:".style(owo_colors::Style::new().bold()), name);
    print_success(&format!("Habit '{}' created successfully", name));

    Ok(())
}