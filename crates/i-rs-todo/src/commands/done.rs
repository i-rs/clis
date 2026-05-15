use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_done(name: String) -> Result<()> {
    let todo = crate::service::toggle_todo_done(&name)?;

    if todo.is_done {
        print_success(&format!("✓ Todo '{}' marked as done", name.green()));
    } else {
        print_success(&format!("✓ Todo '{}' marked as pending", name.green()));
    }

    Ok(())
}
