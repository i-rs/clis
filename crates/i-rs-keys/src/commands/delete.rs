use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    crate::service::delete_key(&name)?;
    print_success(&format!("✓ Key '{}' deleted successfully", name.green()));
    Ok(())
}
