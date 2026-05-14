use crate::models::{AllergyEntry, AllergyStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<AllergyStore> {
    let mut storage = Storage::<AllergyStore>::new("allergy");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &AllergyStore) -> anyhow::Result<()> {
    let storage = Storage::<AllergyStore>::new("allergy");
    storage.save_data(store)
}


pub fn add_entry(store: &mut AllergyStore, entry: AllergyEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut AllergyStore, id: &str) -> Option<AllergyEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a AllergyStore, id: &str) -> Option<&'a AllergyEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a AllergyStore, tag: Option<&'a str>) -> Vec<&'a AllergyEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}