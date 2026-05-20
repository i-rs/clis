use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;

pub fn handle_delete(key: String, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;

    let entry = crate::service::get_kv(&store, &key)?;
    crate::service::delete_kv(&mut store, &key)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Key '{}' deleted successfully", key));
    Ok(())
}
