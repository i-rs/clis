use crate::models::BirthdayRow;
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

pub fn format_table(birthdays: &[&crate::models::Birthday]) -> String {
    let rows: Vec<BirthdayRow> = birthdays
        .iter()
        .map(|b| BirthdayRow::from_birthday(b))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_birthday_count(count: usize) {
    println!("\n{} {} birthdays", "Total:".dimmed(), count.to_string().cyan());
}
