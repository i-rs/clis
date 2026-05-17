use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use crate::service;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(
    id: String,
    content: Option<String>,
    source: Option<Option<String>>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = service::update_spark(&mut store, &id, content, source, tag, remark)?;
    storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Spark '{}' updated", id.green()));

    Ok(())
}
