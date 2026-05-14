use crate::models::{BedEntry, BedStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<BedStore> {
    let mut storage = Storage::<BedStore>::new("bed");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &BedStore) -> anyhow::Result<()> {
    let storage = Storage::<BedStore>::new("bed");
    storage.save_data(store)
}


pub fn add_entry(store: &mut BedStore, entry: BedEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut BedStore, id: &str) -> Option<BedEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a BedStore, id: &str) -> Option<&'a BedEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a BedStore, tag: Option<&'a str>) -> Vec<&'a BedEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}