use crate::models::{Todo, TodoRow};
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(todos: &[&Todo]) -> String {
    let rows: Vec<TodoRow> = todos
        .iter()
        .map(|t| TodoRow::from_todo(t))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_todo_count(pending: usize, done: usize) {
    println!(
        "\n{} {} pending, {} {} done",
        "Total:".dimmed(),
        pending.to_string().cyan(),
        done.to_string().green(),
        "done".dimmed()
    );
}