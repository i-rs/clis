use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(id: String, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let record = crate::service::delete_mood(&mut store, id.clone())?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&record);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!(
        "✓ Record {} deleted: {} {}",
        id.green(),
        record.mood,
        record.mood.label()
    ));
    Ok(())
}
