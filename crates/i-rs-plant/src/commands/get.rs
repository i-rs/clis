use crate::presentation::OutputFormat;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn get_plant(name: String, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if let Some(plant) = storage::find_plant(&store, &name) {
        match output_format {
            OutputFormat::Json => {
                let json_data = serde_json::json!({
                    "name": plant.name,
                    "species": plant.species,
                    "location": plant.location,
                    "watering_interval_days": plant.watering_interval_days,
                    "last_watered": plant.last_watered.to_rfc3339(),
                    "days_until_next_watering": plant.days_until_next_watering(),
                    "needs_water": plant.needs_water(),
                    "tags": plant.tags,
                    "remark": plant.remark,
                    "created_at": plant.created_at.to_rfc3339(),
                    "updated_at": plant.updated_at.to_rfc3339()
                });
                println!("{}", crate::presentation::output_item(&json_data, output_format));
            }
            OutputFormat::Table | OutputFormat::Default => {
                println!("{}", "Plant Details".cyan().bold());
                println!("  {}: {}", "Name".dimmed(), plant.name.green());
                println!("  {}: {}", "Species".dimmed(), plant.species.green());
                println!("  {}: {}", "Location".dimmed(), plant.location.green());
                println!("  {}: {} days", "Watering Interval".dimmed(), plant.watering_interval_days);
                println!("  {}: {}", "Last Watered".dimmed(), plant.last_watered.format("%Y-%m-%d"));
                println!("  {}: {} days", "Days Until Next Watering".dimmed(), plant.days_until_next_watering());
                if plant.needs_water() {
                    println!("  {}: {}", "Status".dimmed(), "Needs water!".yellow().bold());
                } else {
                    println!("  {}: {}", "Status".dimmed(), "Healthy".green());
                }
                if !plant.tags.is_empty() {
                    println!("  {}: {}", "Tags".dimmed(), plant.tags.join(", ").cyan());
                }
                if !plant.remark.is_empty() {
                    println!("  {}:", "Remarks".dimmed());
                    for r in &plant.remark {
                        println!("    - {r}");
                    }
                }
            }
        }
    } else {
        let error_msg = format!("Plant '{name}' not found");
        if matches!(output_format, OutputFormat::Json) {
            println!("{}", crate::presentation::output_error(&error_msg, "NOT_FOUND", output_format));
        } 
        anyhow::bail!("{error_msg}");
    }

    Ok(())
}
