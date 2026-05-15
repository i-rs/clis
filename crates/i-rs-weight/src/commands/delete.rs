use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(date: String) -> Result<()> {
    crate::service::delete_weight(date.clone())?;
    print_success(&format!("✓ Record for {} deleted", date.green()));
    Ok(())
}
