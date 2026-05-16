use crate::models::{ExerciseRecord, ExerciseRow};
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success, print_warning};
use owo_colors::OwoColorize;
pub fn format_table(records: &[&ExerciseRecord]) -> String {
    let rows: Vec<ExerciseRow> = records
        .iter()
        .map(|r| ExerciseRow::from_record(r))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_count(count: usize) {
    println!(
        "\n{} {} records",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
