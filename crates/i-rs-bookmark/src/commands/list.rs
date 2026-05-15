use crate::models::Bookmark;
use crate::presentation::{format_table, print_bookmark_count, print_warning, output_list, OutputFormat};
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let bookmarks = crate::service::list_bookmarks(tag.clone())?;
    let bookmarks_ref: Vec<&Bookmark> = bookmarks.iter().collect();

    if bookmarks_ref.is_empty() {
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

        let items: Vec<ListItem> = bookmarks_ref.iter().map(|b| ListItem {
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

    let table = format_table(&bookmarks_ref);
    println!("\n{table}");

    print_bookmark_count(bookmarks_ref.len());

    Ok(())
}
