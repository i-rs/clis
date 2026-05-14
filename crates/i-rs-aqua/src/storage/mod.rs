use crate::models::{AquaEntry, AquaStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<AquaStore> {
    let mut storage = Storage::<AquaStore>::new("aqua");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &AquaStore) -> anyhow::Result<()> {
    let storage = Storage::<AquaStore>::new("aqua");
    storage.save_data(store)
}


pub fn add_entry(store: &mut AquaStore, entry: AquaEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut AquaStore, id: &str) -> Option<AquaEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a AquaStore, id: &str) -> Option<&'a AquaEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a AquaStore, tag: Option<&'a str>) -> Vec<&'a AquaEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}