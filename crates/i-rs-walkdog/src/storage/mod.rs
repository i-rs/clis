use crate::models::{WalkdogEntry, WalkdogStore};


i_rs_core::create_store!(WalkdogStore, "walkdog");


pub fn add_entry(store: &mut WalkdogStore, entry: WalkdogEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut WalkdogStore, id: &str) -> Option<WalkdogEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a WalkdogStore, id: &str) -> Option<&'a WalkdogEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a WalkdogStore, tag: Option<&'a str>) -> Vec<&'a WalkdogEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}