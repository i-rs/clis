use crate::models::{DoseEntry, DoseStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<DoseStore> {
    let mut storage = Storage::<DoseStore>::new("dose");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &DoseStore) -> anyhow::Result<()> {
    let storage = Storage::<DoseStore>::new("dose");
    storage.save_data(store)
}


pub fn add_entry(store: &mut DoseStore, entry: DoseEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut DoseStore, id: &str) -> Option<DoseEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a DoseStore, id: &str) -> Option<&'a DoseEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a DoseStore, tag: Option<&'a str>) -> Vec<&'a DoseEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}
