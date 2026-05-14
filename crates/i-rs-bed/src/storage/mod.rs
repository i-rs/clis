use crate::models::{BedEntry, BedStore};


i_rs_core::create_store!(BedStore, "bed");


pub fn add_entry(store: &mut BedStore, entry: BedEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut BedStore, id: &str) -> Option<BedEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a BedStore, id: &str) -> Option<&'a BedEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a BedStore, tag: Option<&'a str>) -> Vec<&'a BedEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}