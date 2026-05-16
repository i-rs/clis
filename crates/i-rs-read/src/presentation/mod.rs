use crate::models::{Book, BookRow};
pub use i_rs_core::presentation::OutputFormat;
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{print_error, print_header, print_success};
use owo_colors::OwoColorize;
pub fn format_table(books: &[&Book]) -> String {
    let rows: Vec<BookRow> = books.iter().map(|b| BookRow::from_book(b)).collect();
    i_rs_core::render_table(&rows)
}
pub fn print_book_count(count: usize) {
    println!("\n{} {} books", "Total:".dimmed(), count.to_string().cyan());
}
