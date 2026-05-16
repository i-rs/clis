use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use crate::service;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    content: String,
    source: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = service::add_spark(&mut store, content.clone(), source, tag, remark)?;
    storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    let preview = if content.len() > 30 {
        format!("{}...", &content[..30])
    } else {
        content
    };
    print_success(&format!("✓ Spark recorded: {}", preview.green()));

    Ok(())
}
