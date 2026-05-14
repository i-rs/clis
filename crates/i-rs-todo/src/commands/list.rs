use crate::models::Todo;
use crate::presentation::{format_table, print_todo_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(_all: bool, pending: bool, done: bool, tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let todos: Vec<&Todo> = match (pending, done, &tag) {
        (_, _, Some(t)) => store.filter_by_tag(t),
        (true, false, None) => store.get_pending_todos(),
        (false, true, None) => store.get_done_todos(),
        _ => store.get_all_todos(),
    };

    if todos.is_empty() {
        if matches!(format, OutputFormat::Json) {
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

    if matches!(format, OutputFormat::Json) {
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

    let table = format_table(&todos);
    println!("\n{}", table);

    print_todo_count(store.pending_count(), store.done_count());

    Ok(())
}