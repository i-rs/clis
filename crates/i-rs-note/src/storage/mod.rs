use crate::models::{Note, NoteStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("notes.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("notes.json")
    }
}

pub fn load_store() -> Result<NoteStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(NoteStore::default())
    }
}

pub fn save_store(store: &NoteStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_note(store: &mut NoteStore, note: Note) {
    store.notes.insert(note.name.clone(), note);
}

pub fn remove_note(store: &mut NoteStore, name: &str) -> Option<Note> {
    store.notes.remove(name)
}

pub fn get_note<'a>(store: &'a NoteStore, name: &str) -> Option<&'a Note> {
    store.notes.get(name)
}

pub fn get_note_mut<'a>(store: &'a mut NoteStore, name: &str) -> Option<&'a mut Note> {
    store.notes.get_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a NoteStore, tag: Option<&str>) -> Vec<&'a Note> {
    if let Some(tag) = tag {
        store
            .notes
            .values()
            .filter(|n| n.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.notes.values().collect()
    }
}
