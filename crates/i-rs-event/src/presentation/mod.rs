use crate::models::{Event, EventRow};
pub use i_rs_core::presentation::{
    OutputFormat, print_error, print_header, print_success, print_warning,
};
use owo_colors::OwoColorize;
pub fn format_events_table(events: &[&Event]) -> String {
    let rows: Vec<EventRow> = events.iter().map(|e| EventRow::from_event(e)).collect();
    i_rs_core::render_table(&rows)
}
pub fn print_event_count(count: usize) {
    println!(
        "\n{} {} events",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
