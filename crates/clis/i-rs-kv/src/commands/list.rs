use crate::models::{KvRow, ListItem};
use crate::presentation::{OutputFormat, format_table, output_list, print_entry_count};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_list(
    tag: Option<String>,
    pattern: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    format: OutputFormat,
) -> Result<()> {
    let store = crate::storage::load_store()?;
    let mut entries = crate::service::list_kv(&store, tag.clone(), pattern.as_deref())?;

    let total = entries.len();
    if limit.is_some() || offset.is_some() {
        let start = offset.unwrap_or(0).min(total);
        let end = limit.map(|l| (start + l).min(total)).unwrap_or(total);
        entries = entries[start..end].to_vec();
    }
    let shown = entries.len();

    i_rs_core::handle_empty!(entries, format, tag.as_deref(), "No entries found.");

    if format.is_json() {
        let items: Vec<ListItem> = entries.iter().map(ListItem::from).collect();
        println!(
            "{}",
            output_list(&items, total, tag.as_deref(), format)
        );
        return Ok(());
    }

    let rows: Vec<KvRow> = entries.iter().map(KvRow::from_entry).collect();
    let table = format_table(&rows);
    println!("\n{table}");

    if shown < total {
        println!(
            "  {} {}-{} / {}",
            "Showing:".dimmed(),
            offset.unwrap_or(0) + 1,
            offset.unwrap_or(0) + shown,
            total
        );
    }
    print_entry_count(shown);

    Ok(())
}
