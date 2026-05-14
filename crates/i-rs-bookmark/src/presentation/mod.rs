use crate::models::BookmarkRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(bookmarks: &[&crate::models::Bookmark]) -> String {
    let rows: Vec<BookmarkRow> = bookmarks
        .iter()
        .map(|b| BookmarkRow::from_bookmark(b))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_bookmark_count(count: usize) {
    println!("\n{} {} bookmarks", "Total:".dimmed(), count.to_string().cyan());
}