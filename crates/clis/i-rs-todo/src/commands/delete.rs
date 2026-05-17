use crate::presentation::{OutputFormat, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::delete_todo(&mut store, &name)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        println!(
            "{}",
            serde_json::json!({"success": true, "message": format!("Todo '{}' deleted", name)})
        );
        return Ok(());
    }

    print_success(&format!("✓ Todo '{}' deleted", name.green()));
    Ok(())
}
