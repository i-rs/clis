use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use crate::service;
use crate::storage;
use anyhow::Result;

pub fn handle_delete(id: String, format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = service::get_spark(&store, &id)?;
    service::delete_spark(&mut store, &id)?;
    storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Spark '{}' deleted", id));
    Ok(())
}
