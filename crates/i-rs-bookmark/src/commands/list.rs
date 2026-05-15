use crate::presentation::{format_table, print_bookmark_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let bookmarks: Vec<&crate::models::Bookmark> = storage::filter_by_tag(&store, tag.as_deref());

    if bookmarks.is_empty() {
        if format.is_json() {
            println!("{}", output_list::<serde_json::Value>(&[], 0, tag.as_deref(), format));
        } else {
            print_warning("No bookmarks found.");
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

        let items: Vec<ListItem> = bookmarks.iter().map(|b| ListItem {
            name: b.name.clone(),
            url: b.url.clone(),
            account: b.account.clone(),
            tags: b.tags.clone(),
            remark: b.remark.clone(),
            created_at: b.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: b.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }).collect();

        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&bookmarks);
    println!("\n{table}");

    print_bookmark_count(bookmarks.len());

    Ok(())
}