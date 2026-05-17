use crate::models::SparkRow;
pub use i_rs_core::presentation::output::{output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success};
use owo_colors::OwoColorize;
pub fn format_table(rows: &[SparkRow]) -> String {
    i_rs_core::render_table(rows)
}
pub fn print_entry_count(count: usize) {
    println!(
        "\n{} {} sparks",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
