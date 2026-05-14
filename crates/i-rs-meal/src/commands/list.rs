use crate::models::{ListItem, MealRow};
use crate::presentation::{format_table, print_entry_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::Utc;

pub fn handle_list(date: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if let Some(date_str) = date {
        let parsed_date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .or_else(|_| chrono::NaiveDate::parse_from_str(&date_str, "%Y/%m/%d"))
            .or_else(|_| chrono::NaiveDate::parse_from_str(&date_str, "%d-%m-%Y"))
            .or_else(|_| chrono::NaiveDate::parse_from_str(&date_str, "%d/%m/%Y"))
            .map_err(|_| anyhow::anyhow!("Invalid date format: {date_str}. Use YYYY-MM-DD"))?;

        let entries: Vec<&crate::models::MealEntry> = storage::get_entries_by_date(&store, parsed_date);

        if entries.is_empty() {
            if matches!(format, OutputFormat::Json) {
                println!("{}", output_list::<serde_json::Value>(&[], 0, Some(&date_str), format));
            } else {
                print_warning(&format!("No meals on {date_str}"));
            }
            return Ok(());
        }

        if matches!(format, OutputFormat::Json) {
            let items: Vec<ListItem> = entries.iter().map(|e| ListItem::from(*e)).collect();
            println!("{}", output_list(&items, items.len(), Some(&date_str), format));
            return Ok(());
        }

        let rows: Vec<MealRow> = entries.iter().map(|e| MealRow::from_entry(e)).collect();
        let table = format_table(&rows);
        println!("\n{table}");
        print_entry_count(entries.len());
        return Ok(());
    }

    let today = Utc::now().date_naive();
    let entries: Vec<&crate::models::MealEntry> = storage::get_entries_by_date(&store, today);

    if entries.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(&[], 0, None, format));
        } else {
            print_warning("No meals recorded today.");
        }
        return Ok(());
    }

    if matches!(format, OutputFormat::Json) {
        let items: Vec<ListItem> = entries.iter().map(|e| ListItem::from(*e)).collect();
        println!("{}", output_list(&items, items.len(), None, format));
        return Ok(());
    }

    let rows: Vec<MealRow> = entries.iter().map(|e| MealRow::from_entry(e)).collect();
    let table = format_table(&rows);
    println!("\n{table}");
    print_entry_count(entries.len());

    Ok(())
}
