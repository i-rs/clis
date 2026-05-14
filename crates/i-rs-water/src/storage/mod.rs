use crate::models::{WaterEntry, WaterStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<WaterStore> {
    let mut storage = Storage::<WaterStore>::new("water");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &WaterStore) -> anyhow::Result<()> {
    let storage = Storage::<WaterStore>::new("water");
    storage.save_data(store)
}


pub fn add_entry(store: &mut WaterStore, entry: WaterEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut WaterStore, id: &str) -> Option<WaterEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a WaterStore, id: &str) -> Option<&'a WaterEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a WaterStore, tag: Option<&'a str>) -> Vec<&'a WaterEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}