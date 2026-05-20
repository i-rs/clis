use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;

pub fn handle_update(id: String, weight: Option<f64>, tag: Option<Vec<String>>, remark: Option<Vec<String>>, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let record = crate::service::update_weight(&mut store, id.clone(), weight, tag, remark)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        println!("{}", output_item(&ListItem::from(&record), format));
        return Ok(());
    }

    print_success(&format!("✓ Record {} updated", id));
    Ok(())
}
