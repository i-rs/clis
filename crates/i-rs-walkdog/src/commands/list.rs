use crate::models::{WalkdogRow, ListItem};
use crate::presentation::{format_table, print_entry_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entries: Vec<&crate::models::WalkdogEntry> = storage::filter_by_tag(&store, tag.as_deref());

    if entries.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(&[], 0, tag.as_deref(), format));
        } else {
            print_warning("No records found.");
        }
        return Ok(());
    }

    if matches!(format, OutputFormat::Json) {
        let items: Vec<ListItem> = entries.iter().map(|e| ListItem::from(*e)).collect();
        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let rows: Vec<WalkdogRow> = entries.iter().map(|e| WalkdogRow::from_entry(e)).collect();
    let table = format_table(&rows);
    println!("\n{}", table);

    print_entry_count(entries.len());

    Ok(())
}