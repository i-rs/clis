use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    title: Option<String>,
    priority: Option<String>,
    tag: Option<Vec<String>>,
    content: Option<Vec<String>>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let todo = crate::service::update_todo(&mut store, name.clone(), title, priority, tag, content)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct UpdateOutput {
            name: String,
            title: Option<String>,
            priority: String,
            is_done: bool,
            tags: Vec<String>,
            content: Vec<String>,
        }
        let output = UpdateOutput {
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

    print_success(&format!("✓ Todo '{}' updated", name.green()));
    Ok(())
}
