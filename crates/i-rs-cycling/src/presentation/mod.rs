use crate::models::{CyclingRecord, CyclingRow};
use owo_colors::OwoColorize;

pub use i_rs_core::presentation::{print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};

pub fn format_table(records: &[&CyclingRecord]) -> String {
    let rows: Vec<CyclingRow> = records
        .iter()
        .map(|r| CyclingRow::from_record(r))
        .collect();

    i_rs_core::render_table(&rows)
}

pub fn format_detail_table(record: &CyclingRecord) -> String {
    use crate::models::CyclingDetailRow;
    let rows = CyclingDetailRow::from_record(record);

    i_rs_core::render_table(&rows)
}

pub fn print_record_count(count: usize) {
    println!("\n{} {} records", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_stats(store: &crate::models::CyclingStore) {
    println!("\n{}", "Statistics:".bold().cyan());

    let total_records = store.records_count();
    let total_distance = store.total_distance();
    let total_duration = store.total_duration();
    let total_elevation = store.total_elevation();

    println!("  {:15} {} records", "Records:".dimmed(), total_records.to_string().green());
    println!("  {:15} {:.2} km", "Total Distance:".dimmed(), total_distance);
    println!("  {:15} {} min ({:.1} h)", "Total Duration:".dimmed(), total_duration, total_duration as f64 / 60.0);
    println!("  {:15} {:.0} m", "Total Elevation:".dimmed(), total_elevation);

    if let Some(avg_speed) = store.avg_speed_all() {
        println!("  {:15} {:.2} km/h", "Avg Speed:".dimmed(), avg_speed);
    }

    if total_distance > 0.0 && total_duration > 0 {
        let avg_distance = total_distance / total_records as f64;
        let avg_duration = total_duration as f64 / total_records as f64;
        println!("  {:15} {:.2} km", "Avg Distance:".dimmed(), avg_distance);
        println!("  {:15} {:.0} min", "Avg Duration:".dimmed(), avg_duration);
    }
}
