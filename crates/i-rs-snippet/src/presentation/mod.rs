use crate::models::SnippetRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(snippets: &[&crate::models::Snippet]) -> String {
    let rows: Vec<SnippetRow> = snippets
        .iter()
        .map(|s| SnippetRow::from_snippet(s))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_snippet_count(count: usize) {
    println!("\n{} {} snippets", "Total:".dimmed(), count.to_string().cyan());
}
