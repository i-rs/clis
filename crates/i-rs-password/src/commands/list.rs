use crate::presentation::{format_table, print_entry_count, print_warning};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>) -> Result<()> {
    let store = storage::load_store()?;

    let entries: Vec<&crate::models::PasswordEntry> = storage::filter_by_tag(&store, tag.as_deref());

    if entries.is_empty() {
        print_warning("No entries found.");
        return Ok(());
    }

    let table = format_table(&entries);
    println!("\n{}", table);

    print_entry_count(entries.len());

    Ok(())
}
