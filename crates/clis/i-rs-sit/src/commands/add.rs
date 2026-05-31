use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use crate::service;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    duration_minutes: i32,
    tag: Vec<String>,
    remark: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = service::add_sit(&mut store, duration_minutes, tag, remark)?;
    storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!(
        "✓ Recorded {} minutes of sitting",
        duration_minutes.green()
    ));

    Ok(())
}
