use crate::models::{Investment, InvestmentRow};
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_header, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};
pub fn format_table(investments: &[&Investment]) -> String {
    let rows: Vec<InvestmentRow> = investments
        .iter()
        .map(|i| InvestmentRow::from_investment(i))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_investment_count(count: usize) {
    println!("\n{} {} investments", "Total:".dimmed(), count.to_string().cyan());
}
