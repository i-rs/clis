use crate::models::GiftRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};
pub fn format_table(gifts: &[&crate::models::Gift]) -> String {
    let rows: Vec<GiftRow> = gifts
        .iter()
        .map(|g| GiftRow::from_gift(g))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_gift_count(count: usize) {
    println!("\n{} {} gifts", "Total:".dimmed(), count.to_string().cyan());
}
