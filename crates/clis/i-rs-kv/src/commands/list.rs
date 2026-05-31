use crate::models::{KvRow, ListItem};
use crate::presentation::{OutputFormat, format_table, output_list, print_entry_count};
use anyhow::Result;

pub fn handle_list(
    tag: Option<String>,
    pattern: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = crate::storage::load_store()?;
    let entries = crate::service::list_kv(&store, tag.clone(), pattern.as_deref())?;

    i_rs_core::handle_empty!(entries, format, tag.as_deref(), "No entries found.");

    if format.is_json() {
        let items: Vec<ListItem> = entries.iter().map(ListItem::from).collect();
        println!(
            "{}",
            output_list(&items, items.len(), tag.as_deref(), format)
        );
        return Ok(());
    }

    let rows: Vec<KvRow> = entries.iter().map(KvRow::from_entry).collect();
    let table = format_table(&rows);
    println!("\n{table}");

    print_entry_count(entries.len());

    Ok(())
}
