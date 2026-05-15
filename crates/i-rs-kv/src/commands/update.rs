use crate::models::ListItem;
use crate::presentation::{output_item, print_success, OutputFormat};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(key: String, value: Option<String>, tag: Option<Vec<String>>, remark: Option<Vec<String>>, format: OutputFormat) -> Result<()> {
    let entry = crate::service::update_kv(key.clone(), value, tag, remark)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Key '{}' updated successfully", key.green()));

    Ok(())
}
