use crate::models::KvRow;
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success};
use owo_colors::OwoColorize;
pub fn format_table(rows: &[KvRow]) -> String {
    i_rs_core::render_table(rows)
}
pub fn print_entry_count(count: usize) {
    println!(
        "\n{} {} entries",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
