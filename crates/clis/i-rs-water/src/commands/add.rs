use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use crate::service;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    amount_ml: i32,
    tag: Vec<String>,
    remark: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = service::add_water(&mut store, amount_ml, tag, remark)?;
    storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!(
        "✓ Recorded {}ml at {}",
        amount_ml.green(),
        chrono::Utc::now().format("%H:%M").to_string().cyan()
    ));

    Ok(())
}
