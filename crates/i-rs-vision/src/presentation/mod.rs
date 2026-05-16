use crate::models::{VisionRecord, VisionRow};
pub use i_rs_core::presentation::output::{output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_success, print_warning};
use owo_colors::OwoColorize;
pub fn format_table(records: &[&VisionRecord]) -> String {
    let rows: Vec<VisionRow> = records.iter().map(|r| VisionRow::from_record(r)).collect();
    i_rs_core::render_table(&rows)
}
pub fn print_record_count(count: usize) {
    println!(
        "\n{} {} records",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
