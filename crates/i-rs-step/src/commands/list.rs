use crate::models::{ListItem, StepRow, Summary};
use crate::presentation::{format_table, print_entry_count, print_header, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_list(format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entries: Vec<&crate::models::StepEntry> = store.entries.values().collect();

    if matches!(format, OutputFormat::Json) {
        if entries.is_empty() {
            println!("{}", output_list::<serde_json::Value>(&[], 0, None, format));
            return Ok(());
        }
        let items: Vec<ListItem> = entries.iter().map(|e| ListItem::from(*e)).collect();
        println!("{}", output_list(&items, items.len(), None, format));
        return Ok(());
    }

    let summary = Summary::from(&store);
    println!();
    print_header("Summary");
    println!();
    println!("  {:12} {} steps", "Total:".cyan(), summary.total_steps.to_string().green());
    if let Some(dist) = summary.total_distance {
        println!("  {:12} {:.1} km", "Distance:".cyan(), dist);
    }
    println!("  {:12} {} days", "Days:".cyan(), summary.days_count);
    println!();

    if entries.is_empty() {
        print_warning("No records found.");
        return Ok(());
    }

    let rows: Vec<StepRow> = entries.iter().map(|e| StepRow::from_entry(e)).collect();
    let table = format_table(&rows);
    println!("{table}");

    print_entry_count(entries.len());

    Ok(())
}
