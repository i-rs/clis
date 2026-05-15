use crate::models::{KvRow, ListItem};
use crate::presentation::{format_table, print_entry_count, print_warning, output_list, OutputFormat};
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let entries = crate::service::list_kv(tag.clone())?;
    let entries_ref: Vec<&crate::models::KvEntry> = entries.iter().collect();

    if entries_ref.is_empty() {
        if format.is_json() {
            println!("{}", output_list::<serde_json::Value>(&[], 0, tag.as_deref(), format));
        } else {
            print_warning("No entries found.");
        }
        return Ok(());
    }

    if format.is_json() {
        let items: Vec<ListItem> = entries_ref.iter().map(|e| ListItem::from(*e)).collect();
        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let rows: Vec<KvRow> = entries_ref.iter().map(|e| KvRow::from_entry(e)).collect();
    let table = format_table(&rows);
    println!("\n{table}");

    print_entry_count(entries_ref.len());

    Ok(())
}
