use crate::models::{ListItem, PigRow};
use crate::presentation::{OutputFormat, format_table, output_list, print_entry_count};
use crate::service;
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entries = service::list_pigs(&store, tag.as_deref())?;

    i_rs_core::handle_empty!(entries, format, tag.as_deref());

    if format.is_json() {
        let items: Vec<ListItem> = entries.iter().map(ListItem::from).collect();
        println!(
            "{}",
            output_list(&items, items.len(), tag.as_deref(), format)
        );
        return Ok(());
    }

    let rows: Vec<PigRow> = entries.iter().map(PigRow::from_entry).collect();
    let table = format_table(&rows);
    println!("\n{table}");

    print_entry_count(entries.len());

    Ok(())
}
