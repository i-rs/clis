use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;

pub fn water_plant(name: String, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    let plant_name = name.clone();
    let interval_days;
    {
        let plant = if let Some(p) = store.get_entry_mut(&name) {
            p
        } else {
            let error_msg = format!("Plant '{name}' not found");
            anyhow::bail!("{error_msg}");
        };

        plant.last_watered = Utc::now();
        plant.updated_at = Utc::now();
        interval_days = plant.watering_interval_days;
    }

    storage::save_store(&store)?;

    match output_format {
        OutputFormat::Json => {
            let plant = store
                .get_entry(&plant_name)
                .expect("plant existence validated above");
            println!(
                "{}",
                crate::presentation::output_item(
                    &serde_json::json!({
                        "name": plant.name,
                        "last_watered": plant.last_watered.to_rfc3339(),
                        "next_watering_in_days": plant.days_until_next_watering(),
                        "message": format!("Plant '{}' watered", name)
                    }),
                    output_format
                )
            );
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_success(&format!(
                "Plant '{plant_name}' watered! Next watering in {interval_days} days"
            ));
        }
    }

    Ok(())
}
