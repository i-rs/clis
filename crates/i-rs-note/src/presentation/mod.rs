use crate::models::NoteRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(notes: &[&crate::models::Note]) -> String {
    let rows: Vec<NoteRow> = notes
        .iter()
        .map(|n| NoteRow::from_note(n))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_note_count(count: usize) {
    println!("\n{} {} notes", "Total:".dimmed(), count.to_string().cyan());
}