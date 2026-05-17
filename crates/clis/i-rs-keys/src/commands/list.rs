use crate::models::{KeyRow, ListItem};
use crate::presentation::{OutputFormat, format_table, output_list, print_entry_count};
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let entries = crate::service::list_keys(&store, tag.clone())?;
    let entries_ref: Vec<&crate::models::KeyEntry> = entries.iter().collect();

    i_rs_core::handle_empty!(entries_ref, format, tag.as_deref(), "No entries found.");

    if format.is_json() {
        let items: Vec<ListItem> = entries_ref.iter().map(|e| ListItem::from(*e)).collect();
        println!(
            "{}",
            output_list(&items, items.len(), tag.as_deref(), format)
        );
        return Ok(());
    }

    let rows: Vec<KeyRow> = entries_ref.iter().map(|e| KeyRow::from_entry(e)).collect();
    let table = format_table(&rows);
    println!("\n{table}");

    print_entry_count(entries_ref.len());

    Ok(())
}
