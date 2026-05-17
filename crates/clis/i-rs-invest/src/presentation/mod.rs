use crate::models::{Investment, InvestmentRow};
pub use i_rs_core::presentation::output::{output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header};
use owo_colors::OwoColorize;
pub fn format_table(investments: &[&Investment]) -> String {
    let rows: Vec<InvestmentRow> = investments
        .iter()
        .map(|i| InvestmentRow::from_investment(i))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_investment_count(count: usize) {
    println!(
        "\n{} {} investments",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
