use crate::models::PasswordRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(entries: &[&crate::models::PasswordEntry]) -> String {
    let rows: Vec<PasswordRow> = entries
        .iter()
        .map(|e| PasswordRow::from_entry(e))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_entry_count(count: usize) {
    println!("\n{} {} entries", "Total:".dimmed(), count.to_string().cyan());
}