use crate::models::MealRow;
pub use i_rs_core::presentation::output::{output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success, print_warning};
use owo_colors::OwoColorize;
pub fn format_table(rows: &[MealRow]) -> String {
    i_rs_core::render_table(rows)
}
pub fn print_entry_count(count: usize) {
    println!(
        "\n{} {} meals",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
