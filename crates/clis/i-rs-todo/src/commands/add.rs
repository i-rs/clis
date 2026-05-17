use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    title: Option<String>,
    priority: Option<String>,
    tag: Vec<String>,
    content: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let todo = crate::service::add_todo(&mut store, name.clone(), title, priority, tag, content)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct AddOutput {
            name: String,
            title: Option<String>,
            priority: String,
            is_done: bool,
            tags: Vec<String>,
            content: Vec<String>,
        }
        let output = AddOutput {
            name: todo.name.clone(),
            title: todo.title.clone(),
            priority: todo.priority.label().to_string(),
            is_done: todo.is_done,
            tags: todo.tags.clone(),
            content: todo.content.clone(),
        };
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Todo '{}' added successfully", name.green()));
    Ok(())
}
