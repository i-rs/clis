use crate::models::DeployRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::output_list;
pub fn format_table(rows: &[DeployRow]) -> String {
    i_rs_core::render_table(&rows)
}
pub fn print_deploy_count(count: usize) {
    println!("\n{} {} records", "Total:".dimmed(), count.to_string().cyan());
}
