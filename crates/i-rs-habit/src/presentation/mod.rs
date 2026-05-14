use crate::models::HabitRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_header, print_success, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};
pub fn format_table(rows: &[HabitRow]) -> String {
    i_rs_core::render_table(rows)
}
pub fn print_habit_count(count: usize) {
    println!("\n{} {} habits", "Total:".dimmed(), count.to_string().cyan());
}