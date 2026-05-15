use crate::presentation::{print_success, OutputFormat};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(key: String, format: OutputFormat) -> Result<()> {
    crate::service::delete_kv(&key)?;

    if format.is_json() {
        return Ok(());
    }

    print_success(&format!("✓ Key '{}' deleted successfully", key.green()));

    Ok(())
}
