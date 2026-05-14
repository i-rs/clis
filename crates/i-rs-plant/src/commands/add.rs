use crate::models::Plant;
use crate::presentation::{print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use i_rs_core::validate_name;

pub fn add_plant(
    name: String,
    species: String,
    location: String,
    watering_interval_days: u32,
    tags: Vec<String>,
    remark: Vec<String>,
    output_format: OutputFormat,
) -> Result<()> {
    if let Err(e) = validate_name(&name) {
        anyhow::bail!("{}", e.message);
    }

    let mut store = storage::load_store()?;

    if storage::find_plant(&store, &name).is_some() {
        anyhow::bail!("Plant '{name}' already exists");
    }

    let mut plant = Plant::new(name.clone(), species, location, watering_interval_days);
    plant.tags = tags;
    plant.remark = remark;

    storage::add_plant(&mut store, plant);
    storage::save_store(&store)?;

    match output_format {
        OutputFormat::Json => {
            println!("{}", crate::presentation::output_item(
                &serde_json::json!({"message": format!("Plant '{}' added", name)}),
                output_format
            ));
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_success(&format!("Plant '{name}' added successfully"));
        }
    }

    Ok(())
}
