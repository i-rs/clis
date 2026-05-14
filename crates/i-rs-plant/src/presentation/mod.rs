use crate::models::{Plant, PlantRow, PlantStats};
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_success, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(plants: &[&Plant]) -> String {
    let rows: Vec<PlantRow> = plants
        .iter()
        .map(|p| PlantRow::from_plant(p))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_plant_count(count: usize) {
    println!("\n{} {} plants", "Total:".dimmed(), count.to_string().cyan());
}
pub fn print_stats(stats: &PlantStats) {
    println!("\n{}", "Plant Statistics".cyan().bold());
    println!("  {}: {}", "Total Plants".dimmed(), stats.total_plants.to_string().green());
    println!("  {}: {}", "Needs Water".dimmed(), stats.needs_water.to_string().yellow());
    println!("  {}: {}", "Healthy".dimmed(), stats.healthy.to_string().green());
}
