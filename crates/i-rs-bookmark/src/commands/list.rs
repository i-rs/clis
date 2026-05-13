use crate::presentation::{format_table, print_bookmark_count, print_warning};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>) -> Result<()> {
    let store = storage::load_store()?;

    let bookmarks: Vec<&crate::models::Bookmark> = storage::filter_by_tag(&store, tag.as_deref());

    if bookmarks.is_empty() {
        print_warning("No bookmarks found.");
        return Ok(());
    }

    let table = format_table(&bookmarks);
    println!("\n{}", table);

    print_bookmark_count(bookmarks.len());

    Ok(())
}
