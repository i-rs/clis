use crate::models::Note;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    title: Option<String>,
    tag: Vec<String>,
    content: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.notes.contains_key(&name) {
        print_error(&format!("Note '{}' already exists", name));
        anyhow::bail!("Note '{}' already exists", name);
    }

    let now = Utc::now();
    let note = Note {
        name: name.clone(),
        title,
        tags: tag,
        content,
        created_at: now,
        updated_at: now,
    };

    storage::add_note(&mut store, note);
    storage::save_store(&store)?;

    print_success(&format!("✓ Note '{}' added successfully", name.green()));

    Ok(())
}
