use crate::models::{ContactRow, ListItem};
use crate::presentation::{format_table, print_contact_count, OutputFormat, output_list};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let contacts: Vec<&crate::models::Contact> = if let Some(tag_filter) = &tag {
        store.entries.values().filter(|c| c.tags.contains(tag_filter)).collect()
    } else {
        store.entries.values().collect()
    };

    if format == OutputFormat::Json {
        let items: Vec<ListItem> = contacts.iter().map(|c| (*c).into()).collect();
        output_list(&items, items.len(), tag.as_deref(), format);
    } else {
        if contacts.is_empty() {
            println!("{}", "No contacts found.".cyan());
            return Ok(());
        }

        let rows: Vec<ContactRow> = contacts
            .iter()
            .map(|c| ContactRow::from_contact(c))
            .collect();

        println!("{}", format_table(&rows));
        print_contact_count(rows.len());
    }

    Ok(())
}
