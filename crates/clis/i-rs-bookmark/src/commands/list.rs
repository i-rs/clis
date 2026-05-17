use crate::models::Bookmark;
use crate::presentation::{OutputFormat, format_table, output_list, print_bookmark_count};
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let bookmarks = crate::service::list_bookmarks(&store, tag.clone())?;
    let bookmarks_ref: Vec<&Bookmark> = bookmarks.iter().collect();

    i_rs_core::handle_empty!(bookmarks_ref, format, tag.as_deref(), "No bookmarks found.");

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

        let items: Vec<ListItem> = bookmarks_ref
            .iter()
            .map(|b| ListItem {
                name: b.name.clone(),
                url: b.url.clone(),
                account: b.account.clone(),
                tags: b.tags.clone(),
                remark: b.remark.clone(),
                created_at: b.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: b.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect();

        println!(
            "{}",
            output_list(&items, items.len(), tag.as_deref(), format)
        );
        return Ok(());
    }

    let table = format_table(&bookmarks_ref);
    println!("\n{table}");

    print_bookmark_count(bookmarks_ref.len());

    Ok(())
}
