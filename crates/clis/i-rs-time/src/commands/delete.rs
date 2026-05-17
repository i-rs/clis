use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_delete(id: String, format: OutputFormat) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    let entry = storage::delete_entry(&mut store, &id)?;
    storage::save_store(&store)?;

    if format == OutputFormat::Json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "data": {
                    "id": entry.id,
                    "name": entry.name,
                    "duration_minutes": entry.duration_minutes
                }
            })
        );
    } else {
        print_success(&format!("Deleted entry '{}'", entry.name.cyan()));
        println!(
            "  {} {}",
            "Duration:".cyan(),
            format!("{} minutes", entry.duration_minutes).green()
        );
    }

    Ok(())
}
