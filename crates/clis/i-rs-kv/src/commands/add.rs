use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    key: String,
    value: String,
    tag: Vec<String>,
    remark: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let entry = crate::service::add_kv(&mut store, key.clone(), value, tag, remark)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Key '{}' added successfully", key.green()));

    Ok(())
}
