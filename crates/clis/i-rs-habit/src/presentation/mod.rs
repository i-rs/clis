use crate::models::HabitRow;
pub use i_rs_core::presentation::output::{output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success};
use owo_colors::OwoColorize;
pub fn format_table(rows: &[HabitRow]) -> String {
    i_rs_core::render_table(rows)
}
pub fn print_habit_count(count: usize) {
    println!(
        "\n{} {} habits",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
