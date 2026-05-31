use crate::models::{ListItem, MealRow};
use crate::presentation::{
    OutputFormat, format_table, output_list, print_entry_count, print_warning,
};
use crate::service;
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

        let entries = service::list_meals(&store, Some(parsed_date))?;

        if entries.is_empty() {
            if format.is_json() {
                println!(
                    "{}",
                    output_list::<serde_json::Value>(&[], 0, Some(&date_str), format)
                );
            } else {
                print_warning(&format!("No meals on {date_str}"));
            }
            return Ok(());
        }

        if format.is_json() {
            let items: Vec<ListItem> = entries.iter().map(|e| ListItem::from(e)).collect();
            println!(
                "{}",
                output_list(&items, items.len(), Some(&date_str), format)
            );
            return Ok(());
        }

        let rows: Vec<MealRow> = entries.iter().map(|e| MealRow::from_entry(e)).collect();
        let table = format_table(&rows);
        println!("\n{table}");
        print_entry_count(entries.len());
        return Ok(());
    }

    let today = Utc::now().date_naive();
    let entries = service::list_meals(&store, Some(today))?;

    i_rs_core::handle_empty!(entries, format, None, "No meals recorded today.");

    if format.is_json() {
        let items: Vec<ListItem> = entries.iter().map(|e| ListItem::from(e)).collect();
        println!("{}", output_list(&items, items.len(), None, format));
        return Ok(());
    }

    let rows: Vec<MealRow> = entries.iter().map(|e| MealRow::from_entry(e)).collect();
    let table = format_table(&rows);
    println!("\n{table}");
    print_entry_count(entries.len());

    Ok(())
}
