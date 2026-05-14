use crate::models::{Plant, PlantRow, PlantStats};
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_success, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

pub fn format_table(plants: &[&Plant]) -> String {
    let rows: Vec<PlantRow> = plants
        .iter()
        .map(|p| PlantRow::from_plant(p))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
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
