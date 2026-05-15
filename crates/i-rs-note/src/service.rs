use crate::models::Note;
use crate::storage;
use anyhow::{Context, Result};
use chrono::Utc;

/// List notes, optionally filtered by tag.
pub fn list_notes(tag: Option<String>) -> Result<Vec<Note>> {
    let store = storage::load_store()?;
    let notes: Vec<Note> = if let Some(ref tag_filter) = tag {
        store
            .notes
            .values()
            .filter(|n| n.tags.contains(tag_filter))
            .cloned()
            .collect()
    } else {
        store.notes.values().cloned().collect()
    };
    Ok(notes)
}

/// Get a single note by name.
pub fn get_note(name: &str) -> Result<Note> {
    let store = storage::load_store()?;
    store
        .get_entry(name)
        .cloned()
        .with_context(|| format!("Note '{name}' not found"))
}

/// Add a new note.
pub fn add_note(
    name: String,
    title: Option<String>,
    tags: Vec<String>,
    content: Vec<String>,
) -> Result<Note> {
    let mut store = storage::load_store()?;

    if store.notes.contains_key(&name) {
        anyhow::bail!("Note '{name}' already exists");
    }

    let now = Utc::now();
    let note = Note {
        name: name.clone(),
        title,
        tags,
        content,
        remark: Vec::new(),
        created_at: now,
        updated_at: now,
    };

    store.add_entry(note.clone());
    storage::save_store(&store)?;
    Ok(note)
}

/// Update a note.
pub fn update_note(
    name: String,
    title: Option<String>,
    tags: Option<Vec<String>>,
    content: Option<Vec<String>>,
) -> Result<Note> {
    let mut store = storage::load_store()?;

    let note = store
        .get_entry_mut(&name)
        .with_context(|| format!("Note '{name}' not found"))?;

    if let Some(t) = title {
        note.title = Some(t);
    }
    if let Some(t) = tags {
        note.tags = t;
    }
    if let Some(c) = content {
        note.content = c;
    }
    note.updated_at = Utc::now();

    let updated = note.clone();
    storage::save_store(&store)?;
    Ok(updated)
}

/// Delete a note by name.
pub fn delete_note(name: &str) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.remove_entry(name).is_none() {
        anyhow::bail!("Note '{name}' not found");
    }

    storage::save_store(&store)?;
    Ok(())
}
