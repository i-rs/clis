use crate::models::{LedgerRow, ListItem, Summary};
use crate::presentation::{format_table, print_entry_count, print_header, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_list(category: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entries: Vec<&crate::models::LedgerEntry> = storage::filter_by_category(&store, category.as_deref());

    if format.is_json() {
        if entries.is_empty() {
            println!("{}", output_list::<serde_json::Value>(&[], 0, category.as_deref(), format));
            return Ok(());
        }
        let items: Vec<ListItem> = entries.iter().map(|e| ListItem::from(*e)).collect();
        println!("{}", output_list(&items, items.len(), category.as_deref(), format));
        return Ok(());
    }

    let summary = Summary::from(&store);
    println!();
    print_header("Summary");
    println!();
    println!("  {:12} {}{}", "Income:".cyan(), summary.total_income, summary.currency);
    println!("  {:12} {}{}", "Expense:".red(), summary.total_expense, summary.currency);
    let balance_str = if summary.balance >= 0.0 {
        format!("{}", summary.balance)
    } else {
        format!("{}", summary.balance)
    };
    println!("  {:12} {}{}", "Balance:".cyan(), balance_str, summary.currency);
    println!();

    if entries.is_empty() {
        print_warning("No entries found.");
        return Ok(());
    }

    let rows: Vec<LedgerRow> = entries.iter().map(|e| LedgerRow::from_entry(e)).collect();
    let table = format_table(&rows);
    println!("{table}");

    print_entry_count(entries.len());

    Ok(())
}
