use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    crate::service::delete_note(&name)?;
    print_success(&format!("✓ Note '{}' deleted successfully", name.green()));
    Ok(())
}
