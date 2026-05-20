use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(
    id: String,
    date: Option<String>,
    mood: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let record = crate::service::update_mood(&mut store, id.clone(), date, mood, tag, remark)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&record);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!(
        "✓ Record {} updated: {} {}",
        id.green(),
        record.mood,
        record.mood.label()
    ));
    Ok(())
}
