use crate::models::{GroceryRow, ListItem};
use crate::presentation::{format_table, output_list, print_entry_count, OutputFormat};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_list(tag: Option<String>, purchased: Option<bool>, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let mut items: Vec<&crate::models::GroceryItem> = if let Some(tag_filter) = &tag {
        store.entries.values().filter(|i| i.tags.contains(tag_filter)).collect()
    } else {
        store.entries.values().collect()
    };

    if let Some(p) = purchased {
        items.retain(|item| item.purchased == p);
    }

    if format == OutputFormat::Json {
        let list_items: Vec<ListItem> = items.iter().map(|i| (*i).into()).collect();
        output_list(&list_items, list_items.len(), tag.as_deref(), format);
    } else {
        if items.is_empty() {
            println!("{}", "No grocery items found.".cyan());
            return Ok(());
        }

        let rows: Vec<GroceryRow> = items
            .iter()
            .map(|i| GroceryRow::from_item(i))
            .collect();

        println!("{}", format_table(&rows));
        print_entry_count(rows.len());
    }

    Ok(())
}