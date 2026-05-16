use crate::models::{CalRow, ListItem};
use crate::presentation::{OutputFormat, format_table, output_list, print_entry_count};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entries: Vec<&crate::models::CalEntry> = storage::filter_by_tag(&store, tag.as_deref());

    i_rs_core::handle_empty!(entries, format, tag.as_deref());

    if format.is_json() {
        let items: Vec<ListItem> = entries.iter().map(|e| ListItem::from(*e)).collect();
        println!(
            "{}",
            output_list(&items, items.len(), tag.as_deref(), format)
        );
        return Ok(());
    }

    let rows: Vec<CalRow> = entries.iter().map(|e| CalRow::from_entry(e)).collect();
    let table = format_table(&rows);
    println!("\n{table}");

    print_entry_count(entries.len());

    Ok(())
}
