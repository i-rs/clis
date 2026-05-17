use crate::presentation::{OutputFormat, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(date: String, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::delete_mood(&mut store, date.clone())?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        println!(
            "{}",
            serde_json::json!({"success": true, "message": format!("Record for '{}' deleted", date)})
        );
        return Ok(());
    }

    print_success(&format!("✓ Record for {} deleted", date.green()));
    Ok(())
}
