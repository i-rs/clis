use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    crate::service::delete_bookmark(&name)?;
    print_success(&format!("✓ Bookmark '{}' deleted successfully", name.green()));
    Ok(())
}
