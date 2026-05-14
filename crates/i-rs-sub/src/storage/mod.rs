use crate::models::{SubEntry, SubStore};


i_rs_core::create_store!(SubStore, "sub");


pub fn add_entry(store: &mut SubStore, entry: SubEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut SubStore, name: &str) -> Option<SubEntry> {
    store.remove_entry(name)
}

pub fn get_entry<'a>(store: &'a SubStore, name: &str) -> Option<&'a SubEntry> {
    store.get_entry(name)
}

pub fn get_entry_mut<'a>(store: &'a mut SubStore, name: &str) -> Option<&'a mut SubEntry> {
    store.get_entry_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a SubStore, tag: Option<&'a str>) -> Vec<&'a SubEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}
