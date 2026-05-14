use crate::models::{TimeEntryRow, ListItem};
use crate::presentation::{format_table, print_entry_count, OutputFormat, output_list};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let entries: Vec<_> = if let Some(tag_filter) = &tag {
        storage::list_entries_by_tag(&store, tag_filter)
    } else {
        storage::list_entries(&store)
    };

    if entries.is_empty() {
        if format == OutputFormat::Json {
            println!("{}", output_list::<ListItem>(&[], 0, tag.as_deref(), format));
        } else {
            println!("{}", "No time entries found.".cyan());
        }
        return Ok(());
    }

    if format == OutputFormat::Json {
        let items: Vec<ListItem> = entries.iter().map(|e| (*e).into()).collect();
        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
    } else {
        let rows: Vec<TimeEntryRow> = entries
            .iter()
            .map(|e| TimeEntryRow::from_entry(e))
            .collect();

        println!("{}", format_table(&rows));
        print_entry_count(rows.len());
    }

    Ok(())
}
