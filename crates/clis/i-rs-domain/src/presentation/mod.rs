use crate::models::DomainRow;
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success};
use owo_colors::OwoColorize;
pub fn format_table(domains: &[&crate::models::Domain]) -> String {
    let rows: Vec<DomainRow> = domains.iter().map(|d| DomainRow::from_domain(d)).collect();
    i_rs_core::render_table(&rows)
}
pub fn print_domain_count(count: usize) {
    println!(
        "\n{} {} domains",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
