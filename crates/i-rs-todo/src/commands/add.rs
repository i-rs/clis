use crate::models::{Priority, Todo};
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    title: Option<String>,
    priority: Option<String>,
    tag: Vec<String>,
    content: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.todos.contains_key(&name) {
        anyhow::bail!("Todo '{name}' already exists");
    }

    let priority = match priority {
        Some(p) => Priority::from_str(&p).unwrap_or(Priority::Medium),
        None => Priority::Medium,
    };

    let now = Utc::now();
    let todo = Todo {
        name: name.clone(),
        title,
        priority,
        tags: tag,
        content,
        is_done: false,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(todo);
    storage::save_store(&store)?;

    print_success(&format!("✓ Todo '{}' added successfully", name.green()));

    Ok(())
}
