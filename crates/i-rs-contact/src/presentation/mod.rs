use crate::models::ContactRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_header, print_success, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};
pub fn format_table(rows: &[ContactRow]) -> String {
    i_rs_core::render_table(rows)
}
pub fn print_contact_count(count: usize) {
    println!("\n{} {} contacts", "Total:".dimmed(), count.to_string().cyan());
}
