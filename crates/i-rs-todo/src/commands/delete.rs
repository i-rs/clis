use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    crate::service::delete_todo(&name)?;
    print_success(&format!("✓ Todo '{}' deleted", name.green()));
    Ok(())
}
