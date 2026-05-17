use crate::models::BookmarkRow;
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success};
use owo_colors::OwoColorize;
pub fn format_table(bookmarks: &[&crate::models::Bookmark]) -> String {
    let rows: Vec<BookmarkRow> = bookmarks
        .iter()
        .map(|b| BookmarkRow::from_bookmark(b))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_bookmark_count(count: usize) {
    println!(
        "\n{} {} bookmarks",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
