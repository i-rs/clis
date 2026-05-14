use crate::presentation::{format_table, print_note_count, print_warning};
use crate::presentation::output::{output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let notes: Vec<&crate::models::Note> = storage::filter_by_tag(&store, tag.as_deref());

    if notes.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(&[], 0, tag.as_deref(), format));
        } else {
            print_warning("No notes found.");
        }
        return Ok(());
    }

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            title: Option<String>,
            tags: Vec<String>,
            content: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let items: Vec<ListItem> = notes.iter().map(|n| ListItem {
            name: n.name.clone(),
            title: n.title.clone(),
            tags: n.tags.clone(),
            content: n.content.clone(),
            created_at: n.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: n.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }).collect();

        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&notes);
    println!("\n{}", table);

    print_note_count(notes.len());

    Ok(())
}