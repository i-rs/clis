use crate::models::{Note, NoteStore};
use anyhow::{Context, Result};
use chrono::Utc;

/// List notes, optionally filtered by tag.
pub fn list_notes(store: &NoteStore, tag: Option<String>) -> Result<Vec<Note>> {
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
pub fn get_note(store: &NoteStore, name: &str) -> Result<Note> {
    store
        .get_entry(name)
        .cloned()
        .with_context(|| format!("Note '{name}' not found"))
}

/// Add a new note.
pub fn add_note(
    store: &mut NoteStore,
    name: String,
    title: Option<String>,
    tags: Vec<String>,
    content: Vec<String>,
) -> Result<Note> {

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
    Ok(note)
}

/// Update a note.
pub fn update_note(
    store: &mut NoteStore,
    name: String,
    title: Option<String>,
    tags: Option<Vec<String>>,
    content: Option<Vec<String>>,
) -> Result<Note> {

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
    Ok(updated)
}

/// Delete a note by name.
pub fn delete_note(store: &mut NoteStore, name: &str) -> Result<()> {
    if store.remove_entry(name).is_none() {
        anyhow::bail!("Note '{name}' not found");
    }

    Ok(())
}