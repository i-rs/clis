use crate::models::{Event, EventRow};
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub fn format_events_table(events: &[&Event]) -> String {
    let rows: Vec<EventRow> = events
        .iter()
        .map(|e| EventRow::from_event(e))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_event_count(count: usize) {
    println!("\n{} {} events", "Total:".dimmed(), count.to_string().cyan());
}
