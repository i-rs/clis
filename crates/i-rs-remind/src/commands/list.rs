use crate::presentation::{format_table, print_remind_count, print_warning};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>) -> Result<()> {
    let store = storage::load_store()?;

    let reminds: Vec<&crate::models::Remind> = storage::filter_by_tag(&store, tag.as_deref());

    if reminds.is_empty() {
        print_warning("No reminds found.");
        return Ok(());
    }

    let table = format_table(&reminds);
    println!("\n{}", table);

    print_remind_count(reminds.len());

    Ok(())
}
