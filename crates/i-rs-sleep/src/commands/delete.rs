use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use crate::service;

pub fn handle_delete(id: String, format: OutputFormat) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    service::delete_sleep(&mut store, &id)?;
    storage::save_store(&store)?;

    if format.is_json() {
        println!(
            "{}",
            serde_json::json!({"success": true, "message": format!("Sleep record '{}' deleted", id)})
        );
        return Ok(());
    }

    print_success(&format!("Sleep record '{id}' deleted successfully"));

    Ok(())
}
