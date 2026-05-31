use crate::models::ListItem;
use crate::presentation::{OutputFormat, format_table, output_list};
use anyhow::Result;

pub fn handle_search(query: String, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let entries = crate::service::search_kv(&store, &query)?;

    i_rs_core::handle_empty!(entries, format, None::<&str>, "No matching entries found.");

    if format.is_json() {
        let items: Vec<ListItem> = entries.iter().map(ListItem::from).collect();
        println!("{}", output_list(&items, items.len(), None, format));
        return Ok(());
    }

    let rows: Vec<crate::models::KvRow> = entries
        .iter()
        .map(crate::models::KvRow::from_entry)
        .collect();
    let table = format_table(&rows);
    println!("\n{}", table);
    crate::presentation::print_entry_count(entries.len());

    Ok(())
}
