use crate::models::{PigEntry, PigStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<PigStore> {
    let mut storage = Storage::<PigStore>::new("pig");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &PigStore) -> anyhow::Result<()> {
    let storage = Storage::<PigStore>::new("pig");
    storage.save_data(store)
}


pub fn add_entry(store: &mut PigStore, entry: PigEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut PigStore, id: &str) -> Option<PigEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a PigStore, id: &str) -> Option<&'a PigEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a PigStore, tag: Option<&'a str>) -> Vec<&'a PigEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}
