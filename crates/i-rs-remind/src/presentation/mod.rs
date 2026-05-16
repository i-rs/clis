use crate::models::RemindRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_header, print_success, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(reminds: &[&crate::models::Remind]) -> String {
    let rows: Vec<RemindRow> = reminds
        .iter()
        .map(|r| RemindRow::from_remind(r))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_remind_count(count: usize) {
    println!("\n{} {} reminds", "Total:".dimmed(), count.to_string().cyan());
}