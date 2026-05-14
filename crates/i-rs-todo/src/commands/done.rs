use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;

use owo_colors::OwoColorize;

pub fn handle_done(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    let todo = match store.get_entry_mut(&name) {
        Some(t) => t,
        None => {
            anyhow::bail!("Todo '{name}' not found");
        }
    };

    todo.toggle_done();

    if todo.is_done {
        print_success(&format!("✓ Todo '{}' marked as done", name.green()));
    } else {
        print_success(&format!("✓ Todo '{}' marked as pending", name.green()));
    }

    storage::save_store(&store)?;

    Ok(())
}
