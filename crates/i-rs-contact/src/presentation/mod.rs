use crate::models::ContactRow;
pub use i_rs_core::presentation::output::{output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success};
use owo_colors::OwoColorize;
pub fn format_table(rows: &[ContactRow]) -> String {
    i_rs_core::render_table(rows)
}
pub fn print_contact_count(count: usize) {
    println!(
        "\n{} {} contacts",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
