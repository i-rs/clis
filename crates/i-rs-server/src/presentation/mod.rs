use crate::models::ServerRow;
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success};
use owo_colors::OwoColorize;
pub fn format_table(servers: &[&crate::models::Server]) -> String {
    let rows: Vec<ServerRow> = servers.iter().map(|s| ServerRow::from_server(s)).collect();
    i_rs_core::render_table(&rows)
}
pub fn print_server_count(count: usize) {
    println!(
        "\n{} {} servers",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
