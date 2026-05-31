use crate::models::{ListItem, TodoRow};
use crate::presentation::{OutputFormat, format_table, output_list, print_todo_count};
use anyhow::Result;

pub fn handle_list(
    _all: bool,
    pending: bool,
    done: bool,
    tag: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = crate::storage::load_store()?;
    let todos = crate::service::list_todos(&store, pending, done, tag.clone())?;

    if todos.is_empty() {
        i_rs_core::handle_empty!(todos, format, tag.as_deref(), "No todos found.");
        return Ok(());
    }

    if format.is_json() {
        let items: Vec<ListItem> = todos.iter().map(ListItem::from).collect();

        let filter = tag.or_else(|| {
            if pending {
                Some("pending".to_string())
            } else if done {
                Some("done".to_string())
            } else {
                None
            }
        });

        println!(
            "{}",
            output_list(&items, items.len(), filter.as_deref(), format)
        );
        return Ok(());
    }

    let rows: Vec<TodoRow> = todos.iter().map(TodoRow::from_todo).collect();
    let table = format_table(&rows);
    println!("\n{table}");

    // Count from the already-loaded store
    print_todo_count(store.pending_count(), store.done_count());

    Ok(())
}
