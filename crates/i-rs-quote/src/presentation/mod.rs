use crate::models::QuoteRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(quotes: &[&crate::models::Quote]) -> String {
    let rows: Vec<QuoteRow> = quotes
        .iter()
        .map(|q| QuoteRow::from_quote(q))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_quote_count(count: usize) {
    println!("\n{} {} quotes", "Total:".dimmed(), count.to_string().cyan());
}
