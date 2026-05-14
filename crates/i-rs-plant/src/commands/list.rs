use crate::models::Plant;
use crate::presentation::{format_table, print_plant_count, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn list_plants(
    tag_filter: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    let store = storage::load_store()?;

    let plants: Vec<&Plant> = if let Some(ref tag) = tag_filter {
        store.plants.iter()
            .filter(|p| p.tags.iter().any(|t| t == tag))
            .collect()
    } else {
        store.plants.iter().collect()
    };

    let plant_refs: Vec<&Plant> = plants.iter().map(|p| *p).collect();

    match output_format {
        OutputFormat::Json => {
            let json_data: Vec<serde_json::Value> = plants.iter()
                .map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "species": p.species,
                        "location": p.location,
                        "watering_interval_days": p.watering_interval_days,
                        "last_watered": p.last_watered.to_rfc3339(),
                        "days_until_next_watering": p.days_until_next_watering(),
                        "needs_water": p.needs_water(),
                        "tags": p.tags,
                        "remark": p.remark
                    })
                })
                .collect();
            println!("{}", crate::presentation::output_list(&json_data, store.plants.len(), tag_filter.as_deref(), output_format));
        }
        OutputFormat::Table | OutputFormat::Default => {
            if plant_refs.is_empty() {
                println!("No plants found.");
            } else {
                println!("{}", format_table(&plant_refs));
                print_plant_count(plant_refs.len());
            }
        }
    }

    Ok(())
}
