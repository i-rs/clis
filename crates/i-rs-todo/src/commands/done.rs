use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_done(name: String) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let todo = crate::service::toggle_todo_done(&mut store, &name)?;
    crate::storage::save_store(&store)?;

    if todo.is_done {
        print_success(&format!("✓ Todo '{}' marked as done", name.green()));
    } else {
        print_success(&format!("✓ Todo '{}' marked as pending", name.green()));
    }

    Ok(())
}
