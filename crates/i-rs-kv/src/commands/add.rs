use crate::models::ListItem;
use crate::presentation::{output_item, print_success, OutputFormat};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(key: String, value: String, tag: Vec<String>, remark: Vec<String>, format: OutputFormat) -> Result<()> {
    let entry = crate::service::add_kv(key.clone(), value, tag, remark)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Key '{}' added successfully", key.green()));

    Ok(())
}
