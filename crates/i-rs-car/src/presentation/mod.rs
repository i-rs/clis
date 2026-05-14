use crate::models::{Car, CarDetail, CarRow, FuelRecord, FuelRow, MaintenanceRecord, MaintenanceRow, Stats};
use owo_colors::OwoColorize;

pub use i_rs_core::presentation::{print_error, print_success, OutputFormat};
pub use i_rs_core::presentation::output::output_item;

pub fn format_car_table(cars: &[&Car]) -> String {
    let rows: Vec<CarRow> = cars.iter().map(|c| CarRow::from(*c)).collect();

    if rows.is_empty() {
        return String::new();
    }

    i_rs_core::render_table(&rows)
}

pub fn format_fuel_table(records: &[(&FuelRecord, Option<f64>)]) -> String {
    let rows: Vec<FuelRow> = records.iter().map(|(r, prev)| FuelRow::from_record(r, *prev)).collect();

    if rows.is_empty() {
        return String::new();
    }

    i_rs_core::render_table(&rows)
}

pub fn format_maintenance_table(records: &[&MaintenanceRecord]) -> String {
    let rows: Vec<MaintenanceRow> = records.iter().map(|r| MaintenanceRow::from_record(r)).collect();

    if rows.is_empty() {
        return String::new();
    }

    i_rs_core::render_table(&rows)
}

pub fn format_car_detail(detail: &CarDetail) -> String {
    let mut lines = Vec::new();

    lines.push(format!("{}: {}", "Name".cyan().bold(), detail.name));
    lines.push(format!("{}: {}", "License Plate".cyan().bold(), detail.license_plate));
    lines.push(format!("{}: {}", "Brand".cyan().bold(), detail.brand));
    lines.push(format!("{}: {}", "Model".cyan().bold(), detail.model));
    lines.push(format!("{}: {:.0} km", "Mileage".cyan().bold(), detail.mileage));
    lines.push(format!("{}: {}", "Fuel Records".cyan().bold(), detail.fuel_count));
    lines.push(format!("{}: {}", "Maintenance Records".cyan().bold(), detail.maintenance_count));
    lines.push(format!("{}: {:.2}", "Total Fuel Cost".cyan().bold(), detail.total_fuel_cost));
    lines.push(format!("{}: {:.2}", "Total Maintenance Cost".cyan().bold(), detail.total_maintenance_cost));

    if !detail.tags.is_empty() {
        lines.push(format!("{}: {}", "Tags".cyan().bold(), detail.tags.join(", ")));
    }

    if !detail.remark.is_empty() {
        lines.push(format!("{}:", "Remarks".cyan().bold()));
        for r in &detail.remark {
            lines.push(format!("  - {}", r));
        }
    }

    lines.push(format!("{}: {}", "Created".cyan().bold(), detail.created_at));
    lines.push(format!("{}: {}", "Updated".cyan().bold(), detail.updated_at));

    lines.join("\n")
}

pub fn format_stats(stats: &Stats) -> String {
    let mut lines = Vec::new();

    lines.push("=== Vehicle Statistics ===".cyan().bold().to_string());
    lines.push(format!("{}: {}", "Total Cars".cyan().bold(), stats.total_cars));
    lines.push(format!("{}: {}", "Total Fuel Records".cyan().bold(), stats.total_fuel_records));
    lines.push(format!("{}: {}", "Total Maintenance Records".cyan().bold(), stats.total_maintenance_records));
    lines.push(format!("{}: {:.2}", "Total Fuel Cost".cyan().bold(), stats.total_fuel_cost));
    lines.push(format!("{}: {:.2}", "Total Maintenance Cost".cyan().bold(), stats.total_maintenance_cost));
    lines.push(format!("{}: {:.2}", "Total Cost".cyan().bold(), stats.total_fuel_cost + stats.total_maintenance_cost));

    if !stats.by_car.is_empty() {
        lines.push(String::new());
        lines.push("=== By Car ===".cyan().bold().to_string());
        for (car_name, car_stats) in &stats.by_car {
            lines.push(String::new());
            lines.push(car_name.cyan().bold().to_string());
            lines.push(format!("  Mileage: {:.0} km", car_stats.latest_mileage));
            lines.push(format!("  Fuel Records: {}", car_stats.fuel_count));
            lines.push(format!("  Fuel Cost: {:.2}", car_stats.total_fuel_cost));
            lines.push(format!("  Maintenance Records: {}", car_stats.maintenance_count));
            lines.push(format!("  Maintenance Cost: {:.2}", car_stats.total_maintenance_cost));
            lines.push(format!("  Total Cost: {:.2}", car_stats.total_fuel_cost + car_stats.total_maintenance_cost));
        }
    }

    lines.join("\n")
}

pub fn print_car_count(count: usize) {
    println!("\n{} {} cars", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_fuel_count(count: usize) {
    println!("\n{} {} fuel records", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_maintenance_count(count: usize) {
    println!("\n{} {} maintenance records", "Total:".dimmed(), count.to_string().cyan());
}
