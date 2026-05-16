use crate::presentation::{OutputFormat, format_table, output_list, print_entry_count};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entries: Vec<&crate::models::PasswordEntry> =
        storage::filter_by_tag(&store, tag.as_deref());

    i_rs_core::handle_empty!(entries, format, tag.as_deref(), "No entries found.");

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

        let items: Vec<ListItem> = entries
            .iter()
            .map(|e| ListItem {
                name: e.name.clone(),
                url: e.url.clone(),
                account: e.account.clone(),
                tags: e.tags.clone(),
                remark: e.remark.clone(),
                created_at: e.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: e.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect();

        println!(
            "{}",
            output_list(&items, items.len(), tag.as_deref(), format)
        );
        return Ok(());
    }

    let table = format_table(&entries);
    println!("\n{table}");

    print_entry_count(entries.len());

    Ok(())
}
