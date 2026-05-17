use crate::models::QuoteRow;
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success, print_warning};
use owo_colors::OwoColorize;
pub fn format_table(quotes: &[&crate::models::Quote]) -> String {
    let rows: Vec<QuoteRow> = quotes.iter().map(|q| QuoteRow::from_quote(q)).collect();
    i_rs_core::render_table(&rows)
}
pub fn print_quote_count(count: usize) {
    println!(
        "\n{} {} quotes",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
