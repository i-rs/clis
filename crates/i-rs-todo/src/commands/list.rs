use crate::presentation::{format_table, print_todo_count, print_warning, output_list, OutputFormat};
use anyhow::Result;

pub fn handle_list(_all: bool, pending: bool, done: bool, tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let todos = crate::service::list_todos(&store, pending, done, tag.clone())?;

    if todos.is_empty() {
        if format.is_json() {
            let filter = tag.or_else(|| {
                if pending {
                    Some("pending".to_string())
                } else if done {
                    Some("done".to_string())
                } else {
                    None
                }
            });
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter.as_deref(), format));
        } else {
            print_warning("No todos found.");
        }
        return Ok(());
    }

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            title: Option<String>,
            priority: String,
            is_done: bool,
            tags: Vec<String>,
            content: Vec<String>,
        }

        let items: Vec<ListItem> = todos.iter().map(|t| ListItem {
            name: t.name.clone(),
            title: t.title.clone(),
            priority: t.priority.label().to_string(),
            is_done: t.is_done,
            tags: t.tags.clone(),
            content: t.content.clone(),
        }).collect();

        let filter = tag.or_else(|| {
            if pending {
                Some("pending".to_string())
            } else if done {
                Some("done".to_string())
            } else {
                None
            }
        });

        println!("{}", output_list(&items, items.len(), filter.as_deref(), format));
        return Ok(());
    }

    let refs: Vec<&crate::models::Todo> = todos.iter().collect();
    let table = format_table(&refs);
    println!("\n{table}");

    // Count from the already-loaded store
    print_todo_count(store.pending_count(), store.done_count());

    Ok(())
}
