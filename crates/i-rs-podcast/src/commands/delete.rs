use crate::presentation::{print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.podcasts.remove(&name).is_none() {
        anyhow::bail!("Podcast '{}' not found", name);
    }

    storage::save_store(&store)?;

    if matches!(output_format, OutputFormat::Json) {
        println!("{}", serde_json::json!({
            "success": true,
            "message": format!("Podcast '{}' deleted successfully", name)
        }));
    } else {
        print_success(&format!("✓ Podcast '{}' deleted", name.green()));
    }

    Ok(())
}
