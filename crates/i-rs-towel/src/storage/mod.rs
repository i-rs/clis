use crate::models::{TowelEntry, TowelStore};


i_rs_core::create_store!(TowelStore, "towel");


pub fn add_entry(store: &mut TowelStore, entry: TowelEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut TowelStore, id: &str) -> Option<TowelEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a TowelStore, id: &str) -> Option<&'a TowelEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a TowelStore, tag: Option<&'a str>) -> Vec<&'a TowelEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}