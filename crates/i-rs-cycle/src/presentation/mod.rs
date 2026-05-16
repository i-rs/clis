use crate::models::CycleRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_header, print_success, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(rows: &[CycleRow]) -> String {
    i_rs_core::render_table(rows)
}
pub fn print_entry_count(count: usize) {
    println!("\n{} {} records", "Total:".dimmed(), count.to_string().cyan());
}