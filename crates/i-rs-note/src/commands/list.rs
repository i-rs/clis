use crate::models::Note;
use crate::presentation::{OutputFormat, format_table, output_list, print_note_count};
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let notes = crate::service::list_notes(&store, tag.clone())?;
    let notes_ref: Vec<&Note> = notes.iter().collect();

    i_rs_core::handle_empty!(notes_ref, format, tag.as_deref(), "No notes found.");

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            title: Option<String>,
            tags: Vec<String>,
            content: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let items: Vec<ListItem> = notes_ref
            .iter()
            .map(|n| ListItem {
                name: n.name.clone(),
                title: n.title.clone(),
                tags: n.tags.clone(),
                content: n.content.clone(),
                created_at: n.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: n.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect();

        println!(
            "{}",
            output_list(&items, items.len(), tag.as_deref(), format)
        );
        return Ok(());
    }

    let table = format_table(&notes_ref);
    println!("\n{table}");

    print_note_count(notes_ref.len());

    Ok(())
}
