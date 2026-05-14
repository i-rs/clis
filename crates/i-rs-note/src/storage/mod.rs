use crate::models::{Note, NoteStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<NoteStore> {
    let mut storage = Storage::<NoteStore>::new("note");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &NoteStore) -> anyhow::Result<()> {
    let storage = Storage::<NoteStore>::new("note");
    storage.save_data(store)
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
