use crate::models::{EntityRow, ListItem};
use crate::presentation::{format_table, print_entity_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entities: Vec<&crate::models::Entity> = storage::filter_by_tag(&store, tag.as_deref());

    if entities.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(&[], 0, tag.as_deref(), format));
        } else {
            print_warning("No items found.");
        }
        return Ok(());
    }

    if matches!(format, OutputFormat::Json) {
        let items: Vec<ListItem> = entities.iter().map(|e| ListItem::from(*e)).collect();
        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let rows: Vec<EntityRow> = entities.iter().map(|e| EntityRow::from_entity(e)).collect();
    let table = format_table(&rows);
    println!("\n{}", table);

    print_entity_count(entities.len());

    Ok(())
}
