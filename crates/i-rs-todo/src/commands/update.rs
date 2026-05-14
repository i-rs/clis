use crate::models::Priority;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    title: Option<String>,
    priority: Option<String>,
    tag: Option<Vec<String>>,
    content: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let todo = match store.get_todo_mut(&name) {
        Some(t) => t,
        None => {
            print_error(&format!("Todo '{}' not found", name));
            anyhow::bail!("Todo '{}' not found", name);
        }
    };

    if let Some(t) = title {
        todo.title = Some(t);
    }
    if let Some(p) = priority {
        todo.priority = Priority::from_str(&p).unwrap_or(Priority::Medium);
    }
    if let Some(tags) = tag {
        todo.tags = tags;
    }
    if let Some(c) = content {
        todo.content = c;
    }

    todo.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Todo '{}' updated", name.green()));

    Ok(())
}
