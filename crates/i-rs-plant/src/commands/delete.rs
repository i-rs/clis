use crate::presentation::{print_error, print_success, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn delete_plant(name: String, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    if storage::find_plant(&store, &name).is_none() {
        let error_msg = format!("Plant '{}' not found", name);
        print_error(&error_msg);
        anyhow::bail!("{}", error_msg);
    }

    storage::remove_plant(&mut store, &name);
    storage::save_store(&store)?;

    match output_format {
        OutputFormat::Json => {
            println!("{}", crate::presentation::output_item(
                &serde_json::json!({"message": format!("Plant '{}' deleted", name)}),
                output_format
            ));
        }
        OutputFormat::Table => {
            print_success(&format!("Plant '{}' deleted", name));
        }
    }

    Ok(())
}
