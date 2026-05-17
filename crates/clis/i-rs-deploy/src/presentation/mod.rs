use crate::models::DeployRow;
pub use i_rs_core::presentation::output::output_list;
pub use i_rs_core::presentation::{
    OutputFormat, print_error, print_header, print_success, print_warning,
};
use owo_colors::OwoColorize;
pub fn format_table(rows: &[DeployRow]) -> String {
    i_rs_core::render_table(rows)
}
pub fn print_deploy_count(count: usize) {
    println!(
        "\n{} {} records",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
