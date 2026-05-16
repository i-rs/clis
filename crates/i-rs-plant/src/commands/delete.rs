use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use anyhow::Result;

pub fn delete_plant(name: String, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.get_entry(&name).is_none() {
        let error_msg = format!("Plant '{name}' not found");
        anyhow::bail!("{error_msg}");
    }

    store.remove_entry(&name);
    storage::save_store(&store)?;

    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                crate::presentation::output_item(
                    &serde_json::json!({"message": format!("Plant '{}' deleted", name)}),
                    output_format
                )
            );
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_success(&format!("Plant '{name}' deleted"));
        }
    }

    Ok(())
}
