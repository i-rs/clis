use crate::presentation::{OutputFormat, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(key: String, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::delete_kv(&mut store, &key)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        println!(
            "{}",
            serde_json::json!({"success": true, "message": format!("Key '{}' deleted", key)})
        );
        return Ok(());
    }

    print_success(&format!("✓ Key '{}' deleted successfully", key.green()));

    Ok(())
}
