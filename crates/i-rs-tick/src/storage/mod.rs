use crate::models::{TickEntry, TickStore};


i_rs_core::create_store!(TickStore, "tick");


pub fn add_entry(store: &mut TickStore, entry: TickEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut TickStore, id: &str) -> Option<TickEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a TickStore, id: &str) -> Option<&'a TickEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a TickStore, tag: Option<&'a str>) -> Vec<&'a TickEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}
