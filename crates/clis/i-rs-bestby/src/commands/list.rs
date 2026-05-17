use crate::models::{EntityRow, ListItem};
use crate::presentation::{OutputFormat, format_table, output_list, print_entity_count};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entities: Vec<&crate::models::Entity> = storage::filter_by_tag(&store, tag.as_deref());

    i_rs_core::handle_empty!(entities, format, tag.as_deref(), "No items found.");

    if format.is_json() {
        let items: Vec<ListItem> = entities.iter().map(|e| ListItem::from(*e)).collect();
        println!(
            "{}",
            output_list(&items, items.len(), tag.as_deref(), format)
        );
        return Ok(());
    }

    let rows: Vec<EntityRow> = entities.iter().map(|e| EntityRow::from_entity(e)).collect();
    let table = format_table(&rows);
    println!("\n{table}");

    print_entity_count(entities.len());

    Ok(())
}
