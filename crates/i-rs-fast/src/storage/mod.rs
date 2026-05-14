use crate::models::{FastEntry, FastStore};


i_rs_core::create_store!(FastStore, "fast");


pub fn add_entry(store: &mut FastStore, entry: FastEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut FastStore, id: &str) -> Option<FastEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a FastStore, id: &str) -> Option<&'a FastEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a FastStore, tag: Option<&'a str>) -> Vec<&'a FastEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}