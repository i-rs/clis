use crate::models::{FilterEntry, FilterStore};


i_rs_core::create_store!(FilterStore, "filter");


pub fn add_entry(store: &mut FilterStore, entry: FilterEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut FilterStore, id: &str) -> Option<FilterEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a FilterStore, id: &str) -> Option<&'a FilterEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a FilterStore, tag: Option<&'a str>) -> Vec<&'a FilterEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}