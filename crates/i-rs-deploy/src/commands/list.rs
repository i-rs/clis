use crate::models::{DeployRow, ListItem};
use crate::presentation::{format_table, print_deploy_count, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(
    project: Option<String>,
    environment: Option<String>,
    tag: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = storage::load_store()?;

    let entries: Vec<_> = if let Some(ref p) = project {
        storage::filter_by_project(&store, Some(p))
    } else if let Some(ref e) = environment {
        storage::filter_by_environment(&store, Some(e))
    } else if let Some(ref t) = tag {
        storage::filter_by_tag(&store, Some(t))
    } else {
        store.entries.values().collect()
    };

    let mut entries: Vec<_> = entries;
    entries.sort_by_key(|e| std::cmp::Reverse(e.deployed_at));

    i_rs_core::handle_empty!(entries, format, None, "No deploy records found.");

    if format.is_json() {
        let items: Vec<ListItem> = entries.iter().map(|e| ListItem::from(*e)).collect();
        println!("{}", output_list(&items, items.len(), None, format));
        return Ok(());
    }

    let rows: Vec<DeployRow> = entries.iter().map(|e| DeployRow::from_record(e)).collect();
    let table = format_table(&rows);
    println!("\n{table}");

    print_deploy_count(entries.len());

    Ok(())
}
