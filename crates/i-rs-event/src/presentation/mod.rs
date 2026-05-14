use crate::models::{Event, EventRow};
use owo_colors::OwoColorize;
use tabled::{
    settings::{Color, object::Rows, object::Segment, style::BorderColor, style::Style, themes::Colorization},
    Table,
};

pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};

pub fn format_events_table(events: &[&Event]) -> String {
    let rows: Vec<EventRow> = events
        .iter()
        .map(|e| EventRow::from_event(e))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_event_count(count: usize) {
    println!("\n{} {} events", "Total:".dimmed(), count.to_string().cyan());
}
