use crate::presentation::{print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::Utc;

pub fn update_plant(
    name: String,
    species: Option<String>,
    location: Option<String>,
    watering_interval_days: Option<u32>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    output_format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let plant_name = name.clone();
    {
        let plant = if let Some(p) = store.get_entry_mut(&name) { p } else {
            let error_msg = format!("Plant '{name}' not found");
            anyhow::bail!("{error_msg}");
        };

        if let Some(s) = species {
            plant.species = s;
        }
        if let Some(l) = location {
            plant.location = l;
        }
        if let Some(i) = watering_interval_days {
            plant.watering_interval_days = i;
        }
        if let Some(t) = tags {
            plant.tags = t;
        }
        if let Some(r) = remark {
            plant.remark = r;
        }

        plant.updated_at = Utc::now();
    }

    storage::save_store(&store)?;

    let plant = store.get_entry(&plant_name).expect("plant existence validated above");

    match output_format {
        OutputFormat::Json => {
            println!("{}", crate::presentation::output_item(
                &serde_json::json!({
                    "name": plant.name,
                    "species": plant.species,
                    "location": plant.location,
                    "watering_interval_days": plant.watering_interval_days,
                    "last_watered": plant.last_watered.to_rfc3339(),
                    "tags": plant.tags,
                    "remark": plant.remark
                }),
                output_format
            ));
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_success(&format!("Plant '{plant_name}' updated"));
        }
    }

    Ok(())
}
