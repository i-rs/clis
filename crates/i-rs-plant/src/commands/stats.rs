use crate::models::PlantStats;
use crate::presentation::{OutputFormat, print_stats};
use crate::storage;
use anyhow::Result;

pub fn stats(output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let total_plants = store.plants.len();
    let needs_water = store.plants.iter().filter(|p| p.needs_water()).count();
    let healthy = total_plants - needs_water;

    let stats = PlantStats {
        total_plants,
        needs_water,
        healthy,
        total_waterings: 0,
    };

    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                crate::presentation::output_item(&stats, output_format)
            );
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_stats(&stats);
        }
    }

    Ok(())
}
