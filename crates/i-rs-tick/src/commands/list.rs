use crate::models::{ListItem, TickRow};
use crate::presentation::{format_table, print_entry_count, print_header, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entries: Vec<&crate::models::TickEntry> = storage::filter_by_tag(&store, tag.as_deref());

    if matches!(format, OutputFormat::Json) {
        if entries.is_empty() {
            println!("{}", output_list::<serde_json::Value>(&[], 0, tag.as_deref(), format));
            return Ok(());
        }
        let items: Vec<ListItem> = entries.iter().map(|e| ListItem::from(*e)).collect();
        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let summary = crate::models::Summary::from(&store);
    println!();
    print_header("Summary");
    println!();
    println!("  {:12} {}", "Total:".cyan(), summary.total_duration_str.green());
    println!("  {:12} {} entries", "Count:".cyan(), summary.entries_count);
    println!();

    if entries.is_empty() {
        print_warning("No entries found.");
        return Ok(());
    }

    let rows: Vec<TickRow> = entries.iter().map(|e| TickRow::from_entry(e)).collect();
    let table = format_table(&rows);
    println!("{table}");

    print_entry_count(entries.len());

    Ok(())
}
