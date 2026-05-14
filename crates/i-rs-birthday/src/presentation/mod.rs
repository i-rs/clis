use crate::models::BirthdayRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(birthdays: &[&crate::models::Birthday]) -> String {
    let rows: Vec<BirthdayRow> = birthdays
        .iter()
        .map(|b| BirthdayRow::from_birthday(b))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_birthday_count(count: usize) {
    println!("\n{} {} birthdays", "Total:".dimmed(), count.to_string().cyan());
}
