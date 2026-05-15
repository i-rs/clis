use crate::presentation::{print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(key: String, format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    if storage::remove_entry(&mut store, &key).is_none() {
        anyhow::bail!("Key '{key}' not found");
    }

    storage::save_store(&store)?;

    if matches!(format, OutputFormat::Json) {
        return Ok(());
    }

    print_success(&format!("✓ Key '{}' deleted successfully", key.green()));

    Ok(())
}
