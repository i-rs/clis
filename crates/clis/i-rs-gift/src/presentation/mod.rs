use crate::models::GiftRow;
pub use i_rs_core::presentation::output::{output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success, print_warning};
use owo_colors::OwoColorize;
pub fn format_table(gifts: &[&crate::models::Gift]) -> String {
    let rows: Vec<GiftRow> = gifts.iter().map(|g| GiftRow::from_gift(g)).collect();
    i_rs_core::render_table(&rows)
}
pub fn print_gift_count(count: usize) {
    println!("\n{} {} gifts", "Total:".dimmed(), count.to_string().cyan());
}
