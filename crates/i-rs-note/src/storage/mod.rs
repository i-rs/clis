use crate::models::{Note, NoteStore};

i_rs_core::create_store!(NoteStore, "note");

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
