use crate::models::BirthdayRow;
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success, print_warning};
use owo_colors::OwoColorize;
pub fn format_table(birthdays: &[&crate::models::Birthday]) -> String {
    let rows: Vec<BirthdayRow> = birthdays
        .iter()
        .map(|b| BirthdayRow::from_birthday(b))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_birthday_count(count: usize) {
    println!(
        "\n{} {} birthdays",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
