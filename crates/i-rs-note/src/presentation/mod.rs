use crate::models::NoteRow;
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success};
use owo_colors::OwoColorize;
pub fn format_table(notes: &[&crate::models::Note]) -> String {
    let rows: Vec<NoteRow> = notes.iter().map(|n| NoteRow::from_note(n)).collect();
    i_rs_core::render_table(&rows)
}
pub fn print_note_count(count: usize) {
    println!("\n{} {} notes", "Total:".dimmed(), count.to_string().cyan());
}
