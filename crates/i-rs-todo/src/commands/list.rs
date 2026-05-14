use crate::models::Todo;
use crate::presentation::{format_table, print_todo_count, print_warning};
use crate::storage;
use anyhow::Result;

pub fn handle_list(_all: bool, pending: bool, done: bool, tag: Option<String>) -> Result<()> {
    let store = storage::load_store()?;

    let todos: Vec<&Todo> = match (pending, done, tag) {
        (_, _, Some(t)) => store.filter_by_tag(&t),
        (true, false, None) => store.get_pending_todos(),
        (false, true, None) => store.get_done_todos(),
        _ => store.get_all_todos(),
    };

    if todos.is_empty() {
        print_warning("No todos found.");
        return Ok(());
    }

    let table = format_table(&todos);
    println!("\n{}", table);

    print_todo_count(store.pending_count(), store.done_count());

    Ok(())
}
