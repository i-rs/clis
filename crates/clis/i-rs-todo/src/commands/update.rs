use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;

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
        let output = ListItem::from(&todo);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Todo '{}' updated", name));
    Ok(())
}
