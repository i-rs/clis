use crate::models::{CycleEntry, CycleStore};


i_rs_core::create_store!(CycleStore, "cycle");


pub fn add_entry(store: &mut CycleStore, entry: CycleEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut CycleStore, id: &str) -> Option<CycleEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a CycleStore, id: &str) -> Option<&'a CycleEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a CycleStore, tag: Option<&'a str>) -> Vec<&'a CycleEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}