use crate::models::{SitEntry, SitStore};


i_rs_core::create_store!(SitStore, "sit");


pub fn add_entry(store: &mut SitStore, entry: SitEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut SitStore, id: &str) -> Option<SitEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a SitStore, id: &str) -> Option<&'a SitEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a SitStore, tag: Option<&'a str>) -> Vec<&'a SitEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}