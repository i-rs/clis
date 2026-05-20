use crate::presentation::{OutputFormat, print_success};
use anyhow::Result;

pub fn handle_delete(id: String, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::delete_weight(&mut store, id.clone())?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        println!(
            "{}",
            serde_json::json!({"success": true, "message": format!("Record '{}' deleted", id)})
        );
        return Ok(());
    }

    print_success(&format!("✓ Record {} deleted", id));
    Ok(())
}
