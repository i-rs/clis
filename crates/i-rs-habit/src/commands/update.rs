use crate::presentation::print_success;
use crate::storage;

pub fn handle_update(name: String, description: Option<String>, frequency: Option<String>, tags: Option<Vec<String>>, remark: Option<Vec<String>>) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    storage::update_habit(&mut store, &name, description, frequency, tags, remark)?;
    storage::save_store(&store)?;

    print_success(&format!("Habit '{}' updated successfully", name));

    Ok(())
}