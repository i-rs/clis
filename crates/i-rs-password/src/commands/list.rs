use crate::presentation::{format_table, print_entry_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entries: Vec<&crate::models::PasswordEntry> = storage::filter_by_tag(&store, tag.as_deref());

    if entries.is_empty() {
        if format.is_json() {
            println!("{}", output_list::<serde_json::Value>(&[], 0, tag.as_deref(), format));
        } else {
            print_warning("No entries found.");
        }
        return Ok(());
    }

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            url: String,
            account: Option<String>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let items: Vec<ListItem> = entries.iter().map(|e| ListItem {
            name: e.name.clone(),
            url: e.url.clone(),
            account: e.account.clone(),
            tags: e.tags.clone(),
            remark: e.remark.clone(),
            created_at: e.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: e.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }).collect();

        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&entries);
    println!("\n{table}");

    print_entry_count(entries.len());

    Ok(())
}