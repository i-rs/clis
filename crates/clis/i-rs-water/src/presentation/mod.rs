use crate::models::WaterRow;
pub use i_rs_core::presentation::output::{output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success};
use owo_colors::OwoColorize;
pub fn format_table(rows: &[WaterRow]) -> String {
    i_rs_core::render_table(rows)
}
pub fn print_entry_count(count: usize) {
    println!(
        "\n{} {} records",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
