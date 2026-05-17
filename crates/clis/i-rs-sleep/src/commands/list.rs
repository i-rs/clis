use crate::models::{ListItem, SleepRow};
use crate::presentation::{OutputFormat, format_table, output_list, print_record_count};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let records: Vec<&crate::models::SleepRecord> = if let Some(tag_filter) = &tag {
        store
            .entries
            .values()
            .filter(|r| r.tags.contains(tag_filter))
            .collect()
    } else {
        store.entries.values().collect()
    };

    if format == OutputFormat::Json {
        let items: Vec<ListItem> = records.iter().map(|r| (*r).into()).collect();
        output_list(&items, items.len(), tag.as_deref(), format);
    } else {
        if records.is_empty() {
            println!("{}", "No sleep records found.".cyan());
            return Ok(());
        }

        let rows: Vec<SleepRow> = records.iter().map(|r| SleepRow::from_record(r)).collect();

        println!("{}", format_table(&rows));
        print_record_count(rows.len());
    }

    Ok(())
}
