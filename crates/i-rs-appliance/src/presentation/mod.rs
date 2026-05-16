use crate::models::{Appliance, ApplianceRow, MaintenanceRow, MaintenanceRecord};
use owo_colors::OwoColorize;

pub use i_rs_core::presentation::{print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};

pub fn format_table(appliances: &[&Appliance]) -> String {
    let rows: Vec<ApplianceRow> = appliances
        .iter()
        .map(|a| ApplianceRow::from_appliance(a))
        .collect();

    i_rs_core::render_table(&rows)
}

#[allow(dead_code)]
pub fn format_maintenance_table(records: &[&MaintenanceRecord]) -> String {
    let rows: Vec<MaintenanceRow> = records
        .iter()
        .map(|r| MaintenanceRow::from_record(r))
        .collect();

    i_rs_core::render_table(&rows)
}

pub fn print_appliance_count(count: usize) {
    println!("\n{} {} appliances", "Total:".dimmed(), count.to_string().cyan());
}

#[allow(dead_code)]
pub fn print_stats(store: &crate::models::ApplianceStore) {
    let total = store.appliances_count();
    let expired = store.expired_count();
    let soon = store.needs_replacement_count();

    println!("\n{}", "Statistics:".bold().cyan());
    println!("  {:12} {}", "Total:".dimmed(), total.to_string().cyan());
    println!("  {:12} {} (expired)", "Expired:".dimmed(), expired.to_string().red());
    println!("  {:12} {} (within 90 days)", "Replace Soon:".dimmed(), soon.to_string().yellow());
}
