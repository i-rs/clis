use crate::models::{PetbathEntry, PetbathStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<PetbathStore> {
    let mut storage = Storage::<PetbathStore>::new("petbath");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &PetbathStore) -> anyhow::Result<()> {
    let storage = Storage::<PetbathStore>::new("petbath");
    storage.save_data(store)
}


pub fn add_entry(store: &mut PetbathStore, entry: PetbathEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut PetbathStore, id: &str) -> Option<PetbathEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a PetbathStore, id: &str) -> Option<&'a PetbathEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a PetbathStore, tag: Option<&'a str>) -> Vec<&'a PetbathEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}