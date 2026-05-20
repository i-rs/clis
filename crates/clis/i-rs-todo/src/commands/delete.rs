use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;

pub fn handle_delete(name: String, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;

    let todo = crate::service::get_todo(&store, &name)?;
    crate::service::delete_todo(&mut store, &name)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&todo);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Todo '{}' deleted", name));
    Ok(())
}
