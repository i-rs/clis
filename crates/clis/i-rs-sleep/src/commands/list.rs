use crate::models::{ListItem, SleepRow};
use crate::presentation::{OutputFormat, format_table, output_list, print_entry_count};
use crate::service;
use crate::storage;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let entries = service::list_sleeps(&store, tag.as_deref())?;

    i_rs_core::handle_empty!(entries, format, tag.as_deref());

    if format.is_json() {
        let items: Vec<ListItem> = entries.iter().map(ListItem::from).collect();
        output_list(&items, items.len(), tag.as_deref(), format);
    } else {
        let rows: Vec<SleepRow> = entries.iter().map(SleepRow::from_record).collect();
        println!("{}", format_table(&rows));
        print_entry_count(rows.len());
    }

    Ok(())
}
