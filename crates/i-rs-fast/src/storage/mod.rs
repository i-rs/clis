use crate::models::{FastEntry, FastStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<FastStore> {
    let mut storage = Storage::<FastStore>::new("fast");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &FastStore) -> anyhow::Result<()> {
    let storage = Storage::<FastStore>::new("fast");
    storage.save_data(store)
}


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