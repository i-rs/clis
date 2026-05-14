use crate::models::{FilterEntry, FilterStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<FilterStore> {
    let mut storage = Storage::<FilterStore>::new("filter");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &FilterStore) -> anyhow::Result<()> {
    let storage = Storage::<FilterStore>::new("filter");
    storage.save_data(store)
}


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