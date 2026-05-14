use crate::models::DomainRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(domains: &[&crate::models::Domain]) -> String {
    let rows: Vec<DomainRow> = domains
        .iter()
        .map(|d| DomainRow::from_domain(d))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_domain_count(count: usize) {
    println!("\n{} {} domains", "Total:".dimmed(), count.to_string().cyan());
}