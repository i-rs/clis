use crate::presentation::{OutputFormat, print_success};
use crate::service;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(id: String, format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    service::delete_sit(&mut store, &id)?;
    storage::save_store(&store)?;

    if format.is_json() {
        println!(
            "{}",
            serde_json::json!({"success": true, "message": format!("Record '{}' deleted", id)})
        );
        return Ok(());
    }

    print_success(&format!("✓ Record '{}' deleted", id.green()));

    Ok(())
}
