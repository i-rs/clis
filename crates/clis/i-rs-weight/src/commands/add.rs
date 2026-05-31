use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;

pub fn handle_add(
    date: Option<String>,
    weight: f64,
    tag: Vec<String>,
    remark: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let record = crate::service::add_weight(&mut store, date, weight, tag, remark)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        println!("{}", output_item(&ListItem::from(&record), format));
        return Ok(());
    }

    print_success(&format!("✓ Weight {} kg recorded", record.weight));
    Ok(())
}
