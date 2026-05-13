use crate::models::DomainRow;
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub fn print_success(msg: &str) {
    println!("{}", msg.green());
}

pub fn print_header(msg: &str) {
    println!("\n{}", msg.bold().cyan());
}

pub fn print_error(msg: &str) {
    eprintln!("{}", msg.red());
}

pub fn print_warning(msg: &str) {
    println!("{}", msg.yellow());
}

pub fn format_table(domains: &[&crate::models::Domain]) -> String {
    let rows: Vec<DomainRow> = domains
        .iter()
        .map(|d| DomainRow::from_domain(d))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_domain_count(count: usize) {
    println!("\n{} {} domains", "Total:".dimmed(), count.to_string().cyan());
}
