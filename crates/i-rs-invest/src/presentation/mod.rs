use crate::models::{Investment, InvestmentRow};
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_error, print_header, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};

pub fn format_table(investments: &[&Investment]) -> String {
    let rows: Vec<InvestmentRow> = investments
        .iter()
        .map(|i| InvestmentRow::from_investment(i))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_investment_count(count: usize) {
    println!("\n{} {} investments", "Total:".dimmed(), count.to_string().cyan());
}
