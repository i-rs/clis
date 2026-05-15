use crate::models::{Priority, Todo, TodoStore};
use anyhow::{Context, Result};
use chrono::Utc;

/// List todos with optional filters. Returns owned Todos.
pub fn list_todos(store: &TodoStore, pending: bool, done: bool, tag: Option<String>) -> Result<Vec<Todo>> {
    let todos: Vec<&Todo> = match (pending, done, &tag) {
        (_, _, Some(t)) => store.filter_by_tag(t),
        (true, false, None) => store.get_pending_todos(),
        (false, true, None) => store.get_done_todos(),
        _ => store.get_all_todos(),
    };
    Ok(todos.into_iter().cloned().collect())
}

/// Get a single todo by name.
pub fn get_todo(store: &TodoStore, name: &str) -> Result<Todo> {
    let todo = store
        .get_entry(name)
        .cloned()
        .with_context(|| format!("Todo '{name}' not found"))?;
    Ok(todo)
}

/// Add a new todo. Returns the created Todo.
pub fn add_todo(
    store: &mut TodoStore,
    name: String,
    title: Option<String>,
    priority: Option<String>,
    tags: Vec<String>,
    content: Vec<String>,
) -> Result<Todo> {

    if store.todos.contains_key(&name) {
        anyhow::bail!("Todo '{name}' already exists");
    }

    let priority = priority
        .as_deref()
        .and_then(Priority::from_str)
        .unwrap_or(Priority::Medium);

    let now = Utc::now();
    let todo = Todo {
        name: name.clone(),
        title,
        priority,
        tags,
        content,
        is_done: false,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(todo.clone());
    Ok(todo)
}

/// Update an existing todo. Returns the updated Todo.
pub fn update_todo(
    store: &mut TodoStore,
    name: String,
    title: Option<String>,
    priority: Option<String>,
    tags: Option<Vec<String>>,
    content: Option<Vec<String>>,
) -> Result<Todo> {

    let todo = store
        .get_entry_mut(&name)
        .with_context(|| format!("Todo '{name}' not found"))?;

    if let Some(t) = title {
        todo.title = Some(t);
    }
    if let Some(ref p) = priority {
        todo.priority = Priority::from_str(p).unwrap_or(Priority::Medium);
    }
    if let Some(t) = tags {
        todo.tags = t;
    }
    if let Some(c) = content {
        todo.content = c;
    }

    todo.updated_at = Utc::now();
    let updated = todo.clone();
    Ok(updated)
}

/// Delete a todo by name.
pub fn delete_todo(store: &mut TodoStore, name: &str) -> Result<()> {
    if store.remove_entry(name).is_none() {
        anyhow::bail!("Todo '{name}' not found");
    }

    Ok(())
}

/// Toggle a todo's done/pending status. Returns the toggled Todo.
pub fn toggle_todo_done(store: &mut TodoStore, name: &str) -> Result<Todo> {

    let todo = store
        .get_entry_mut(name)
        .with_context(|| format!("Todo '{name}' not found"))?;

    todo.toggle_done();
    let updated = todo.clone();
    Ok(updated)
}
