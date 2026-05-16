use crate::models::SnippetRow;
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success, print_warning};
use owo_colors::OwoColorize;
pub fn format_table(snippets: &[&crate::models::Snippet]) -> String {
    let rows: Vec<SnippetRow> = snippets
        .iter()
        .map(|s| SnippetRow::from_snippet(s))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_snippet_count(count: usize) {
    println!(
        "\n{} {} snippets",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
