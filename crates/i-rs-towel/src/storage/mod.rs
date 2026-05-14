use crate::models::{TowelEntry, TowelStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<TowelStore> {
    let mut storage = Storage::<TowelStore>::new("towel");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &TowelStore) -> anyhow::Result<()> {
    let storage = Storage::<TowelStore>::new("towel");
    storage.save_data(store)
}


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