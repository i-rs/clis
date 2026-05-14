use crate::models::{ExerciseRecord, ExerciseRow};
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_success, print_warning, print_header, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(records: &[&ExerciseRecord]) -> String {
    let rows: Vec<ExerciseRow> = records
        .iter()
        .map(|r| ExerciseRow::from_record(r))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_count(count: usize) {
    println!("\n{} {} records", "Total:".dimmed(), count.to_string().cyan());
}
