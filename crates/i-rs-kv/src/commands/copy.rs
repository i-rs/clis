use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_copy(src: String, dst: String, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let entry = crate::service::copy_kv(&mut store, &src, dst.clone())?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Copied '{}' to '{}'", src.green(), dst.green()));

    Ok(())
}
