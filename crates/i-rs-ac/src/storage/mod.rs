use crate::models::{AcEntry, AcStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<AcStore> {
    let mut storage = Storage::<AcStore>::new("ac");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &AcStore) -> anyhow::Result<()> {
    let storage = Storage::<AcStore>::new("ac");
    storage.save_data(store)
}


pub fn add_entry(store: &mut AcStore, entry: AcEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut AcStore, id: &str) -> Option<AcEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a AcStore, id: &str) -> Option<&'a AcEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a AcStore, tag: Option<&'a str>) -> Vec<&'a AcEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}