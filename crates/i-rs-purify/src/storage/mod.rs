use crate::models::{PurifyEntry, PurifyStore};


i_rs_core::create_store!(PurifyStore, "purify");


pub fn add_entry(store: &mut PurifyStore, entry: PurifyEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut PurifyStore, id: &str) -> Option<PurifyEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a PurifyStore, id: &str) -> Option<&'a PurifyEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a PurifyStore, tag: Option<&'a str>) -> Vec<&'a PurifyEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}