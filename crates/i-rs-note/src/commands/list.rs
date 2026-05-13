use crate::presentation::{format_table, print_note_count, print_warning};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>) -> Result<()> {
    let store = storage::load_store()?;

    let notes: Vec<&crate::models::Note> = storage::filter_by_tag(&store, tag.as_deref());

    if notes.is_empty() {
        print_warning("No notes found.");
        return Ok(());
    }

    let table = format_table(&notes);
    println!("\n{}", table);

    print_note_count(notes.len());

    Ok(())
}
